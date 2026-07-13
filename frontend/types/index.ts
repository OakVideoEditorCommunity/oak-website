export interface ReleaseAsset {
  id: string
  platform: string
  arch?: string | null
  filename: string
  size_bytes?: number | null
  sync_status: string
  synced_at?: string | null
}

export interface Release {
  id: string
  version: string
  tag_name: string
  release_notes?: string | null
  is_prerelease: boolean
  published_at?: string | null
  assets: ReleaseAsset[]
}

export interface DocPageSummary {
  slug: string
  title: string
  lang: string
}

export interface DocPage {
  slug: string
  title: string
  lang: string
  html: string
}

export interface DocsIndex {
  zh: DocPageSummary[]
  en: DocPageSummary[]
}
