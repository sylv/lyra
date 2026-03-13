import { createFileRoute } from '@tanstack/react-router'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../components/ui/tabs'

export const Route = createFileRoute('/settings')({
  component: RouteComponent,
})

function RouteComponent() {
  const buildDate = new Date(__BUILD_TIME__).toLocaleString()
  return <div className="pt-6"> 
        <Tabs defaultValue="about" className="w-full" >
            <TabsList >
            <TabsTrigger  value="about">About</TabsTrigger >
        </TabsList>
        <div className="bg-zinc-400/10 rounded p-3">
        <TabsContent value="about">
            <p className="text-zinc-400 text-sm">
                Based on {__BRANCH__} {__REVISION__}, built on {buildDate}.
            </p>
        </TabsContent>
        </div>
    </Tabs>
  </div>
}
