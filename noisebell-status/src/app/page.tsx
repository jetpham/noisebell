import { Suspense } from 'react'
import { Status } from "@/components/Status"

export const experimental_ppr = true

export default function Home() {
  return (
    <div className="min-h-screen flex flex-col items-center justify-center p-4 gap-6">
      <Suspense fallback={
        <div className="inline-block w-fit min-w-[300px] p-6 bg-gray-50 border-2 border-gray-200 rounded-lg">
          <div className="animate-pulse">
            <div className="h-6 bg-gray-200 rounded mb-2"></div>
            <div className="h-4 bg-gray-200 rounded"></div>
          </div>
        </div>
      }>
        <Status />
      </Suspense>
    </div>
  );
}