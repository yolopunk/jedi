// Extract only explicit model metadata from native protocol records.
export function nativeModel(record: any): string | undefined {
  const candidate =
    record?.model ??
    record?.data?.model ??
    record?.params?.model ??
    record?.params?.thread?.model ??
    record?.message?.model ??
    record?.part?.model ??
    record?.info?.modelID
  const name =
    typeof candidate === 'string'
      ? candidate
      : (candidate?.id ?? candidate?.name ?? candidate?.modelID)
  return typeof name === 'string' && name.trim() && name !== 'unknown' ? name : undefined
}
