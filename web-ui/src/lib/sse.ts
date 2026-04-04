type SSECallback = () => void

let eventSource: EventSource | null = null
const listeners = new Set<SSECallback>()

export function subscribeToUpdates(callback: SSECallback): () => void {
  listeners.add(callback)

  if (!eventSource) {
    eventSource = new EventSource(`${window.location.origin}/api/events`)

    eventSource.addEventListener('activity_changed', () => {
      listeners.forEach((cb) => cb())
    })

    eventSource.onerror = () => {
      // EventSource auto-reconnects on error
    }
  }

  return () => {
    listeners.delete(callback)
    if (listeners.size === 0 && eventSource) {
      eventSource.close()
      eventSource = null
    }
  }
}
