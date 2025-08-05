# NoiseBell Status API

A simple status API built with Next.js and Prisma, designed to report whether a noisebridge is open or closed.

## API Endpoints

### GET `/api/status`

Retrieves the current status of the service.

**Response:**

- `200 OK`: Returns the current status
- `200 OK`: Returns `{"status": "closed"}` if no status records exist

**Example Response:**

```json
{
  "status": "OPEN",
  "createdAt": "2024-01-15T10:30:00.000Z"
}
```

### POST `/api/status`

Updates the current status of the service.

**Headers Required:**

```text
Authorization: Bearer YOUR_API_KEY
Content-Type: application/json
```

**Request Body:**

```json
{
  "status": "open" | "closed"
}
```

**Response:**

- `201 Created`: Status successfully updated
- `400 Bad Request`: Invalid JSON, missing status field, or invalid status value
- `401 Unauthorized`: Missing or invalid API key
- `500 Internal Server Error`: Server configuration error

**Example Request:**

```bash
curl -X POST http://localhost:3000/api/status \
  -H "Authorization: Bearer your-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{"status": "open"}'
```

**Example Response:**

```json
{
  "status": "OPEN",
  "createdAt": "2024-01-15T10:30:00.000Z"
}
```

## Environment Variables

### Local Development

Create a `.env.local` file in the root directory with the following variables:

```env
DATABASE_URL="postgresql://username:password@localhost:5432/your_database"
API_KEY="your-secret-api-key-here"
```

### Production with Vercel

If you're deploying to Vercel, you can manage environment variables using Vercel CLI:

1. **Link your project to Vercel:**

   ```bash
   pnpx vercel env link
   ```

2. **Pull environment variables from Vercel:**

   ```bash
   pnpx vercel env pull .env.local
   ```

3. **Set environment variables in Vercel dashboard:**
   - Go to your project in the Vercel dashboard
   - Navigate to Settings → Environment Variables
   - Add the required variables:
     - `DATABASE_URL`: Your PostgreSQL connection string
     - `API_KEY`: Your secret API key

### Required Environment Variables

- `DATABASE_URL`: PostgreSQL connection string
- `API_KEY`: Secret key for API authentication

## Database Schema

The API uses a PostgreSQL database with the following schema:

```sql
-- StatusType enum
CREATE TYPE "StatusType" AS ENUM ('OPEN', 'CLOSED');

-- Status table
CREATE TABLE "statuses" (
  "id" SERIAL PRIMARY KEY,
  "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
  "status" "StatusType" NOT NULL DEFAULT 'OPEN'
);
```

## Development

### Prerequisites

- Node.js 18+
- PostgreSQL database
- pnpm (recommended) or npm

### Installation

1. Clone the repository
2. Install dependencies:

   ```bash
   pnpm install
   ```

3. Set up your environment variables in `.env.local`

4. Run database migrations:

   ```bash
   pnpm prisma db push
   ```

5. Start the development server:

   ```bash
   pnpm dev
   ```

The API will be available at `http://localhost:3000/api/status`

### Database Management

Generate Prisma client:

```bash
pnpm prisma generate
```

View database in Prisma Studio:

```bash
pnpm prisma studio
```

## Error Codes

| Status Code | Description |
|-------------|-------------|
| 200 | Success - Status retrieved |
| 201 | Success - Status created/updated |
| 400 | Bad Request - Invalid input |
| 401 | Unauthorized - Invalid or missing API key |
| 500 | Internal Server Error - Server configuration issue |

## Status Values

The API accepts lowercase values in requests but stores them as uppercase enum values:

- `"open"` → `"OPEN"`
- `"closed"` → `"CLOSED"`

## License

This project is open source and available under the MIT License.
