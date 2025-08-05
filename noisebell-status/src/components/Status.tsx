import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"

async function getStatus(): Promise<'open' | 'closed'> {
  try {
    const response = await fetch(`${process.env.NEXT_PUBLIC_BASE_URL || 'http://localhost:3000'}/api/status`, {
      cache: 'no-store'
    });
    
    if (!response.ok) {
      return 'closed';
    }
    
    const data = await response.json();
    return data?.status === 'open' ? 'open' : 'closed';
  } catch {
    return 'closed';
  }
}

export async function Status() {
  const status = await getStatus();
  
  const statusConfig = {
    open: {
      title: 'Open',
      description: "It's time to start hacking.",
      image: 'https://raw.githubusercontent.com/jetpham/noisebell/refs/heads/webhooks/media/open.png',
      color: 'text-green-600',
      bgColor: 'bg-green-50',
      borderColor: 'border-green-200'
    },
    closed: {
      title: 'Closed',
      description: "We'll see you again soon.",
      image: 'https://raw.githubusercontent.com/jetpham/noisebell/refs/heads/webhooks/media/closed.png',
      color: 'text-red-600',
      bgColor: 'bg-red-50',
      borderColor: 'border-red-200'
    }
  }

  const config = statusConfig[status]

  return (
    <Card className={`${config.bgColor} ${config.borderColor} border-2 inline-block w-fit min-w-[300px]`}>
      <CardHeader className="flex flex-row items-center justify-between">
        <div className="flex-1">
          <CardTitle className={`text-2xl font-bold ${config.color}`}>
            {config.title}
          </CardTitle>
          <CardDescription className="text-base">
            {config.description}
          </CardDescription>
        </div>
        <div className="flex-shrink-0 ml-4 flex items-center">
          {/* eslint-disable-next-line @next/next/no-img-element */}
          <img 
            src={config.image} 
            alt={`A knife switch in the ${config.title} position`}
            className="h-full max-h-[60px] w-auto object-contain"
          />
        </div>
      </CardHeader>
    </Card>
  )
} 