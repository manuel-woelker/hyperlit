/**
 * Set up Server-Sent Events connection for hot-reload notifications.
 *
 * Connects to /api/events and triggers reload callback when files change.
 * Automatically reconnects on connection loss.
 *
 * @param onReload - Callback to trigger when files change
 * @returns Cleanup function to close the connection
 */
export function setupSSE(onReload: () => void): () => void {
    console.log("Establishing SSE connection...")
    try {
        const eventSource = new EventSource('/api/events');

    eventSource.addEventListener('file-changed', (event) => {
        console.log('File changed, reloading...', event.data);
        onReload();
    });

    eventSource.addEventListener('ping', () => {
        // Keep-alive message, no action needed
    });

    eventSource.onerror = (error) => {
        console.error('SSE connection error:', error);
        // EventSource automatically reconnects
    };

    // Return cleanup function
    return () => {
        console.log('Closing SSE connection');
        eventSource.close();
    };
    } catch (e) {
        console.warn("Unable to establish SSE connection");
        console.log(e);
        return ()=> {}
    }
}
