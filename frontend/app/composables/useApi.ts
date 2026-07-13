export function useApi() {
  const config = useRuntimeConfig()

  async function fetchApi<T>(path: string, options: any = {}): Promise<T> {
    // Use the internal base URL on the server (e.g. http://backend:8080)
    // and the public base URL in the browser.
    const baseUrl = import.meta.server
      ? config.apiBaseUrl
      : config.public.apiBaseUrl
    const url = `${baseUrl}${path}`
    return await $fetch<T>(url, {
      ...options,
      credentials: 'omit',
    })
  }

  return { fetchApi }
}
