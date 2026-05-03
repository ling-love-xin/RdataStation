const performanceMetrics = new Map<string, number[]>()
const isDev = import.meta.env.DEV

export interface PerformanceMetric {
  name: string
  duration: number
  timestamp: number
}

export interface PerformanceSummary {
  name: string
  count: number
  avg: number
  min: number
  max: number
  p95: number
}

export function usePerformanceMonitor() {
  function startTimer(name: string): () => number {
    const startTime = performance.now()

    return () => {
      const duration = performance.now() - startTime
      recordMetric(name, duration)
      return duration
    }
  }

  function recordMetric(name: string, duration: number) {
    if (!performanceMetrics.has(name)) {
      performanceMetrics.set(name, [])
    }

    const metrics = performanceMetrics.get(name)!
    metrics.push(duration)

    if (metrics.length > 1000) {
      metrics.shift()
    }

    if (isDev) {
      console.log(`[PERF] ${name}: ${duration.toFixed(2)}ms`)
    }
  }

  function getSummary(name: string): PerformanceSummary | null {
    const metrics = performanceMetrics.get(name)
    if (!metrics || metrics.length === 0) {
      return null
    }

    const sorted = [...metrics].sort((a, b) => a - b)
    const count = sorted.length
    const sum = sorted.reduce((a, b) => a + b, 0)
    const avg = sum / count
    const min = sorted[0]
    const max = sorted[count - 1]
    const p95Index = Math.floor(count * 0.95)
    const p95 = sorted[p95Index]

    return { name, count, avg, min, max, p95 }
  }

  function getAllSummaries(): PerformanceSummary[] {
    const summaries: PerformanceSummary[] = []

    for (const name of performanceMetrics.keys()) {
      const summary = getSummary(name)
      if (summary) {
        summaries.push(summary)
      }
    }

    return summaries.sort((a, b) => b.avg - a.avg)
  }

  function clearMetrics(name?: string) {
    if (name) {
      performanceMetrics.delete(name)
    } else {
      performanceMetrics.clear()
    }
  }

  function logPerformanceReport() {
    if (!isDev) return

    const summaries = getAllSummaries()

    console.group('[PERF] Performance Report')
    console.table(
      summaries.map(s => ({
        Metric: s.name,
        Count: s.count,
        'Avg (ms)': s.avg.toFixed(2),
        'Min (ms)': s.min.toFixed(2),
        'Max (ms)': s.max.toFixed(2),
        'P95 (ms)': s.p95.toFixed(2)
      }))
    )
    console.groupEnd()
  }

  return {
    startTimer,
    recordMetric,
    getSummary,
    getAllSummaries,
    clearMetrics,
    logPerformanceReport
  }
}
