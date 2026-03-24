import { NextResponse } from "next/server";

/**
 * Health Check API Route
 * 
 * Returns the current status of the SuperInstance Ranch backend.
 * Used by the frontend dashboard to verify connectivity.
 * 
 * @returns {Object} JSON response with status and timestamp
 */
export async function GET() {
  try {
    // Basic health check - in production, this would check:
    // - Database connectivity
    // - Model pool status
    // - TensorRT-LLM availability
    
    const healthStatus = {
      status: "healthy",
      message: "SuperInstance Ranch API is running",
      timestamp: new Date().toISOString(),
      version: "0.1.0",
      // These would be real values in production
      uptime: process.uptime(),
      memory: process.memoryUsage(),
    };
    
    return NextResponse.json(healthStatus, { status: 200 });
  } catch (error) {
    // Log the error for debugging
    console.error("[API] Health check failed:", error);
    
    // Return a proper error response
    return NextResponse.json(
      { 
        status: "unhealthy",
        error: error instanceof Error ? error.message : "Unknown error",
        timestamp: new Date().toISOString()
      },
      { status: 500 }
    );
  }
}

/**
 * POST handler for Ranch commands
 * 
 * Accepts commands to control the Ranch (e.g., trigger Night School, create breed).
 * 
 * @param {Request} request - The incoming request with command payload
 */
export async function POST(request: Request) {
  try {
    const body = await request.json();
    
    // Validate request body
    if (!body || !body.command) {
      return NextResponse.json(
        { error: "Missing required field: command" },
        { status: 400 }
      );
    }
    
    const { command, payload } = body;
    
    // Route commands to appropriate handlers
    // In production, these would connect to the Rust backend via WebSocket
    switch (command) {
      case "night_school":
        return NextResponse.json({
          success: true,
          message: "Night School triggered",
          timestamp: new Date().toISOString()
        });
        
      case "create_breed":
        if (!payload?.name || !payload?.species) {
          return NextResponse.json(
            { error: "Missing required fields: name, species" },
            { status: 400 }
          );
        }
        return NextResponse.json({
          success: true,
          message: `Breed '${payload.name}' created for species '${payload.species}'`,
          timestamp: new Date().toISOString()
        });
        
      case "sync_herd":
        return NextResponse.json({
          success: true,
          message: "Herd sync initiated",
          timestamp: new Date().toISOString()
        });
        
      default:
        return NextResponse.json(
          { error: `Unknown command: ${command}` },
          { status: 400 }
        );
    }
  } catch (error) {
    console.error("[API] POST handler error:", error);
    
    return NextResponse.json(
      { 
        error: error instanceof Error ? error.message : "Internal server error",
        timestamp: new Date().toISOString()
      },
      { status: 500 }
    );
  }
}
