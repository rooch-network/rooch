export interface HistogramStat {
  sum: number
  count: number
}

export interface PrunerMetrics {
  currentPhase: number
  sweepExpiredDeleted: HistogramStat
  incrementalSweepDeleted: HistogramStat
  reachableNodesScanned: HistogramStat
  bloomFilterSizeBytes: number
  diskSpaceReclaimedBytes: number
  processingSpeedNodesPerSec: HistogramStat
  errorCount: number
}

type MetricSample = {
  name: string
  labels: Record<string, string>
  value: number
}

export class PrometheusClient {
  constructor(private readonly port: number = 9184) {}

  async fetchMetrics(): Promise<PrunerMetrics> {
    const response = await fetch(`http://localhost:${this.port}/metrics`)
    const text = await response.text()
    const samples = this.parseMetrics(text)

    const phaseSamples = samples.filter((s) => s.name === 'pruner_current_phase')
    const currentPhase = phaseSamples.reduce((acc, sample) => Math.max(acc, sample.value), 0)

    const sweepExpiredDeleted = this.histogram(samples, 'pruner_sweep_nodes_deleted', {
      phase: 'SweepExpired',
    })
    const incrementalSweepDeleted = this.histogram(samples, 'pruner_sweep_nodes_deleted', {
      phase: 'incremental',
    })

    return {
      currentPhase,
      sweepExpiredDeleted,
      incrementalSweepDeleted,
      reachableNodesScanned: this.histogram(samples, 'pruner_reachable_nodes_scanned', {
        phase: 'BuildReach',
      }),
      bloomFilterSizeBytes: this.gauge(samples, 'pruner_bloom_filter_size_bytes'),
      diskSpaceReclaimedBytes: this.counter(samples, 'pruner_disk_space_reclaimed_bytes'),
      processingSpeedNodesPerSec: this.histogram(
        samples,
        'pruner_processing_speed_nodes_per_sec',
      ),
      errorCount: samples
        .filter((s) => s.name === 'pruner_error_count' || s.name === 'pruner_error_count_total')
        .reduce((acc, sample) => acc + sample.value, 0),
    }
  }

  private histogram(
    samples: MetricSample[],
    name: string,
    labelMatch?: Record<string, string>,
  ): HistogramStat {
    const sumSample = this.findSample(samples, `${name}_sum`, labelMatch)
    const countSample = this.findSample(samples, `${name}_count`, labelMatch)
    return {
      sum: sumSample?.value ?? 0,
      count: countSample?.value ?? 0,
    }
  }

  private gauge(samples: MetricSample[], name: string, labelMatch?: Record<string, string>): number {
    const sample =
      this.findSample(samples, name, labelMatch) ||
      this.findSample(samples, `${name}_total`, labelMatch) ||
      samples.find((s) => s.name === name || s.name === `${name}_total`)
    return sample?.value ?? 0
  }

  private counter(
    samples: MetricSample[],
    name: string,
    labelMatch?: Record<string, string>,
  ): number {
    const sample =
      this.findSample(samples, name, labelMatch) ||
      this.findSample(samples, `${name}_total`, labelMatch)
    return sample?.value ?? 0
  }

  private findSample(
    samples: MetricSample[],
    name: string,
    labelMatch?: Record<string, string>,
  ): MetricSample | undefined {
    return samples.find((sample) => {
      if (sample.name !== name) {
        return false
      }
      if (!labelMatch) {
        return true
      }
      for (const [key, value] of Object.entries(labelMatch)) {
        if (sample.labels[key] !== value) {
          return false
        }
      }
      return true
    })
  }

  private parseMetrics(text: string): MetricSample[] {
    const samples: MetricSample[] = []
    const lines = text.split('\n')
    for (const line of lines) {
      if (!line || line.startsWith('#')) {
        continue
      }
      const match =
        line.match(
          /^([a-zA-Z_:][a-zA-Z0-9_:]*)(\{([^}]*)\})?\s+([+-]?(?:\d+\.?\d*|\d*\.\d+)(?:[eE][+-]?\d+)?)/,
        ) ?? undefined
      if (!match) {
        continue
      }

      const name = match[1]
      const labelStr = match[3]
      const value = parseFloat(match[4])

      const labels: Record<string, string> = {}
      if (labelStr) {
        for (const part of labelStr.split(',')) {
          const [k, v] = part.split('=')
          if (k && v) {
            labels[k.trim()] = v.replace(/^"|"$/g, '').trim()
          }
        }
      }
      samples.push({ name, labels, value })
    }
    return samples
  }
}
