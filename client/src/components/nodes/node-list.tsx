import { startTransition, Suspense, useEffect, useState, type FC } from "react";
import type { NodeFilter, NodePageQueryVariables } from "../../@generated/gql/graphql";
import { NodePage } from "./node-page";
import stringify from "fast-json-stable-stringify";

const PER_PAGE = 30;

interface NodeListProps {
  displayKind: DisplayKind;
  filter: NodeFilter;
  perPage?: number;
}

export enum DisplayKind {
  Poster,
  Episode,
}

export const NodeList: FC<NodeListProps> = ({ displayKind, filter, perPage = PER_PAGE }) => {
  const [pageVariables, setPageVariables] = useState<NodePageQueryVariables[]>([
    { after: null, filter, first: perPage },
  ]);

  useEffect(() => {
    startTransition(() => {
      setPageVariables([{ after: null, filter, first: perPage }]);
    });
  }, [stringify(filter), perPage]);

  return (
    <>
      {pageVariables.map((variables, index) => (
        <NodePage
          key={index}
          displayKind={displayKind}
          variables={variables}
          isFirst={index === 0}
          isLast={index === pageVariables.length - 1}
          onLoadMore={(after) => {
            setPageVariables((prev) => [...prev, { after, filter, first: perPage }]);
          }}
        />
      ))}
    </>
  );
};
