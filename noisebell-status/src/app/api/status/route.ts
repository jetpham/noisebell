import { unstable_cacheTag as cacheTag } from 'next/cache'
import { PrismaClient } from '@prisma/client'
import { revalidateTag } from 'next/cache'

type StatusResponse = {
    status: 'OPEN' | 'CLOSED'
    createdAt: string
}

// Create a single Prisma client instance
const globalForPrisma = globalThis as unknown as {
    prisma: PrismaClient | undefined
  }
  
  const prisma = globalForPrisma.prisma ?? new PrismaClient()
  
  if (process.env.NODE_ENV !== 'production') globalForPrisma.prisma = prisma
  
// Cached function that returns a StatusResponse to avoid serialization issues
async function getCachedStatus(): Promise<StatusResponse | null> {
    'use cache'
    cacheTag('status')
    
    const status = await prisma.status.findFirst({
        orderBy: {
            createdAt: 'desc'
        }
    });
    
    if (!status) {
        return null;
    }
    
    return {
        status: status.status,
        createdAt: status.createdAt.toISOString()
    };
}

export async function GET() {
    const cachedResult = await getCachedStatus();
    
    if (!cachedResult) {
        return Response.json({ status: 'closed' });
    }
    
    return Response.json(cachedResult);
}

export async function POST(request: Request) {
    const authHeader = request.headers.get('authorization');
    const apiKey = process.env.API_KEY;
    
    if (!apiKey) {
        console.error('API_KEY environment variable is not set');
        return new Response(JSON.stringify({ error: 'Server configuration error' }), { status: 500 });
    }
    
    if (!authHeader || !authHeader.startsWith('Bearer ')) {
        return new Response(JSON.stringify({ error: 'Missing or invalid Authorization header' }), { status: 401 });
    }
    
    const providedKey = authHeader.substring(7);
    
    if (providedKey !== apiKey) {
        return new Response(JSON.stringify({ error: 'Invalid API key' }), { status: 401 });
    }

    let body;
    try {
        body = await request.json();
    } catch (error) {
        return new Response(JSON.stringify({ error: 'Invalid JSON in request body' }), { status: 400 });
    }
    
    const { status } = body;

    if (!status) {
        return new Response(JSON.stringify({ error: 'Missing status field in request body' }), { status: 400 });
    }

    if (status !== 'open' && status !== 'closed') {
        return new Response(JSON.stringify({ error: 'Invalid status value. Must be "open" or "closed"' }), { status: 400 });
    }

    const statusEnum = status.toUpperCase() as 'OPEN' | 'CLOSED';

    const newStatus = await prisma.status.create({
        data: {
            status: statusEnum,
        },
    });

    revalidateTag('status');
    
    const response: StatusResponse = {
        status: newStatus.status,
        createdAt: newStatus.createdAt.toISOString()
    };
    
    return Response.json(response, { status: 201 });
}
