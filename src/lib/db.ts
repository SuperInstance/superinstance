/**
 * Database Client Singleton
 * 
 * Provides a Prisma client instance with hot-reload support for development.
 * Uses the globalThis pattern to prevent multiple Prisma instances during
 * Next.js development hot reloads.
 * 
 * @module lib/db
 * 
 * @example
 * ```tsx
 * import { db } from '@/lib/db'
 * 
 * const user = await db.user.findFirst({ where: { id: 1 } })
 * ```
 * 
 * @remarks
 * - In development, stores the client on globalThis to survive hot reloads
 * - In production, creates a new client each time
 * - Logs all queries in development for debugging
 * 
 * @see https://www.prisma.io/docs/guides/database/troubleshooting-orm/help-articles/nextjs-prisma-client-dev-practices
 */
import { PrismaClient } from '@prisma/client'

/**
 * Global type augmentation for Prisma client persistence
 * during Next.js development hot reloads
 */
const globalForPrisma = globalThis as unknown as {
  prisma: PrismaClient | undefined
}

/**
 * Prisma database client instance
 * 
 * Reuse existing client in development to prevent connection pool exhaustion
 * from hot reload creating new clients on each rebuild.
 */
export const db =
  globalForPrisma.prisma ??
  new PrismaClient({
    // Log queries in development for debugging
    log: process.env.NODE_ENV === 'production' ? [] : ['query', 'info', 'warn', 'error'],
  })

// Store client globally in development to prevent multiple instances
if (process.env.NODE_ENV !== 'production') {
  globalForPrisma.prisma = db
}
