use anyhow::Result;
use oar_ocr_core::processors::{
    BoxType, CTCLabelDecode, ColorOrder, DBPostProcess, DetResizeForTest, LimitType,
    NormalizeImage, ScoreMode, TensorLayout,
};
use ort::session::{Session, builder::PrepackedWeights};
use std::{
    fs,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    path::Path,
    sync::{Condvar, Mutex},
};

// ---------------------------------------------------------------------------
// Generic session pool
// ---------------------------------------------------------------------------

pub struct Pool<T> {
    sessions: Mutex<Vec<T>>,
    available: Condvar,
}

impl<T> Pool<T> {
    pub fn new(sessions: Vec<T>) -> Self {
        Self {
            sessions: Mutex::new(sessions),
            available: Condvar::new(),
        }
    }

    pub fn acquire(&self) -> PoolGuard<'_, T> {
        let mut guard = self.sessions.lock().unwrap();
        loop {
            if !guard.is_empty() {
                let session = guard.pop().unwrap();
                return PoolGuard {
                    pool: self,
                    item: ManuallyDrop::new(session),
                };
            }
            guard = self.available.wait(guard).unwrap();
        }
    }
}

pub struct PoolGuard<'a, T> {
    pool: &'a Pool<T>,
    item: ManuallyDrop<T>,
}

impl<T> Deref for PoolGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.item
    }
}

impl<T> DerefMut for PoolGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.item
    }
}

impl<T> Drop for PoolGuard<'_, T> {
    fn drop(&mut self) {
        // Safety: value is only taken here, in drop, exactly once.
        let item = unsafe { ManuallyDrop::take(&mut self.item) };
        self.pool.sessions.lock().unwrap().push(item);
        self.pool.available.notify_one();
    }
}

pub struct DetSession {
    pub session: Session,
    pub resizer: DetResizeForTest,
    pub normalizer: NormalizeImage,
    pub postprocessor: DBPostProcess,
}

pub struct RecSession {
    pub session: Session,
    pub decoder: CTCLabelDecode,
}

pub struct OcrPool {
    pub det: Pool<DetSession>,
    pub rec: Pool<RecSession>,
}

impl OcrPool {
    pub fn new(
        det_path: &Path,
        rec_path: &Path,
        dict_path: &Path,
        size: usize,
        weights: &PrepackedWeights,
    ) -> Result<Self> {
        let dict_text = fs::read_to_string(dict_path)?;
        let base_dict: Vec<String> = dict_text.lines().map(|l| l.to_string()).collect();

        let mut det_sessions = Vec::with_capacity(size);
        let mut rec_sessions = Vec::with_capacity(size);

        for _ in 0..size {
            let det_resizer = DetResizeForTest::new(
                None,
                None,
                None,
                Some(960),
                Some(LimitType::Max),
                None,
                Some(4000),
            );
            let det_normalizer = NormalizeImage::with_color_order(
                Some(1.0 / 255.0),
                Some(vec![0.485, 0.456, 0.406]),
                Some(vec![0.229, 0.224, 0.225]),
                Some(TensorLayout::CHW),
                Some(ColorOrder::BGR),
            )?;
            let det_postprocessor = DBPostProcess::new(
                Some(0.3),
                Some(0.6),
                Some(1000),
                Some(2.0),
                Some(false),
                Some(ScoreMode::Fast),
                Some(BoxType::Quad),
            );
            let det_session = Session::builder()
                .map_err(|e| anyhow::anyhow!("{e}"))?
                .with_prepacked_weights(weights)
                .map_err(|e| anyhow::anyhow!("{e}"))?
                .commit_from_file(det_path)?;
            det_sessions.push(DetSession {
                session: det_session,
                resizer: det_resizer,
                normalizer: det_normalizer,
                postprocessor: det_postprocessor,
            });

            let rec_decoder = CTCLabelDecode::from_string_list(Some(&base_dict), true, false);
            let rec_session = Session::builder()
                .map_err(|e| anyhow::anyhow!("{e}"))?
                .with_prepacked_weights(weights)
                .map_err(|e| anyhow::anyhow!("{e}"))?
                .commit_from_file(rec_path)?;
            rec_sessions.push(RecSession {
                session: rec_session,
                decoder: rec_decoder,
            });
        }

        Ok(Self {
            det: Pool::new(det_sessions),
            rec: Pool::new(rec_sessions),
        })
    }
}
