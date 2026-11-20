export type Point = { x: number; y: number }
export type AnnotationTool = 'rect' | 'ellipse' | 'arrow' | 'line' | 'pen' | 'mosaic' | 'text'

export interface Stroke {
  tool: Exclude<AnnotationTool, 'text'>
  color: string
  width: number
  start: Point
  end: Point
  points: Point[]
}

interface TextMark {
  tool: 'text'
  point: Point
  color: string
  size: number
  text: string
}

export type Annotation = Stroke | TextMark
export const MOSAIC_BLOCK = 16
export const TEXT_FONT_FAMILY = '"Segoe UI", "Microsoft YaHei UI", "PingFang SC", sans-serif'
export const TEXT_LINE_HEIGHT = 1.25

function drawMosaic(ctx: CanvasRenderingContext2D, mark: Stroke, source: HTMLCanvasElement) {
  const block = MOSAIC_BLOCK
  const radius = Math.max(block * 0.7, mark.width * 4)
  const cells = new Set<number>()

  function brush(point: Point) {
    const x0 = Math.max(0, Math.floor((point.x - radius) / block))
    const x1 = Math.min(source.width - 1, Math.floor((point.x + radius) / block))
    const y0 = Math.max(0, Math.floor((point.y - radius) / block))
    const y1 = Math.min(source.height - 1, Math.floor((point.y + radius) / block))
    for (let y = y0; y <= y1; y++) {
      for (let x = x0; x <= x1; x++) cells.add(y * source.width + x)
    }
  }

  brush(mark.start)
  for (let i = 1; i < mark.points.length; i++) {
    const from = mark.points[i - 1]
    const to = mark.points[i]
    const steps = Math.ceil(Math.hypot(to.x - from.x, to.y - from.y) / block)
    for (let n = 1; n <= steps; n++) {
      brush({
        x: from.x + ((to.x - from.x) * n) / steps,
        y: from.y + ((to.y - from.y) * n) / steps,
      })
    }
  }

  ctx.save()
  ctx.imageSmoothingEnabled = false
  ctx.fillStyle = '#181b21'
  for (const cell of cells) {
    const x = cell % source.width
    const y = Math.floor(cell / source.width)
    ctx.fillRect(x * block, y * block, block, block)
    ctx.drawImage(source, x, y, 1, 1, x * block, y * block, block, block)
  }
  ctx.restore()
}

export function drawAnnotations(
  ctx: CanvasRenderingContext2D,
  annotations: Annotation[],
  mosaicSource: HTMLCanvasElement | undefined
) {
  ctx.lineCap = 'round'
  ctx.lineJoin = 'round'
  ctx.textBaseline = 'top'
  for (const mark of annotations) {
    if (mark.tool === 'mosaic') {
      if (mosaicSource) drawMosaic(ctx, mark, mosaicSource)
      continue
    }
    ctx.fillStyle = mark.color
    if (mark.tool === 'text') {
      ctx.font = `600 ${mark.size}px ${TEXT_FONT_FAMILY}`
      mark.text.split('\n').forEach((line, index) => {
        ctx.fillText(line, mark.point.x, mark.point.y + index * mark.size * TEXT_LINE_HEIGHT)
      })
      continue
    }
    ctx.strokeStyle = mark.color
    ctx.lineWidth = mark.width
    const { start, end } = mark
    const dx = end.x - start.x
    const dy = end.y - start.y
    ctx.beginPath()
    switch (mark.tool) {
      case 'rect':
        ctx.rect(start.x, start.y, dx, dy)
        break
      case 'ellipse':
        ctx.ellipse(
          start.x + dx / 2,
          start.y + dy / 2,
          Math.abs(dx / 2),
          Math.abs(dy / 2),
          0,
          0,
          Math.PI * 2
        )
        break
      case 'pen':
        ctx.moveTo(start.x, start.y)
        if (mark.points.length === 1) {
          ctx.arc(start.x, start.y, mark.width / 2, 0, Math.PI * 2)
          ctx.fill()
          continue
        }
        for (const point of mark.points) ctx.lineTo(point.x, point.y)
        break
      case 'arrow':
      case 'line':
        ctx.moveTo(start.x, start.y)
        ctx.lineTo(end.x, end.y)
        if (mark.tool === 'arrow') {
          const tip = Math.min(Math.hypot(dx, dy) * 0.45, mark.width * 4)
          const angle = Math.atan2(dy, dx)
          for (const offset of [-Math.PI / 6, Math.PI / 6]) {
            ctx.moveTo(end.x, end.y)
            ctx.lineTo(
              end.x - tip * Math.cos(angle + offset),
              end.y - tip * Math.sin(angle + offset)
            )
          }
        }
        break
    }
    ctx.stroke()
  }
}
