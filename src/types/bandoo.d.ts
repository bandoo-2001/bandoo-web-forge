export {}

declare global {
  interface Window {
    __BANDOO__?: {
      app: {
        id: string
        name: string
        url: string
      }
      permissions: Readonly<Record<string, boolean>>
      version: string
      getTitle: () => string
      getRoute: () => {
        href: string
        pathname: string
        search: string
        hash: string
      }
      notify: (title: string, body?: string) => Promise<boolean>
      notification: {
        send: (title: string, body?: string) => Promise<boolean>
      }
      clipboard: {
        readText: () => Promise<string>
        writeText: (text: string) => Promise<boolean>
      }
      shell: {
        exec: (payload: { command: string; cwd?: string; env?: Record<string, string>; timeoutMs?: number }) => Promise<unknown>
      }
      fs: {
        readText: (path: string) => Promise<unknown>
        writeText: (path: string, text: string) => Promise<unknown>
        readDir: (path: string) => Promise<unknown>
        exists: (path: string) => Promise<unknown>
        mkdir: (path: string) => Promise<unknown>
        remove: (path: string) => Promise<unknown>
      }
      network: {
        fetch: (payload: { url: string; method?: string; headers?: Record<string, string>; body?: string; timeoutMs?: number }) => Promise<unknown>
      }
      page: {
        query: (selector: string) => Element | null
        focus: (selector: string) => boolean
        click: (selector: string) => boolean
        type: (selector: string, text: string) => boolean
      }
      automation: {
        run: (
          actions: Array<{
            kind: string
            selector?: string
            text?: string
            script?: string
            value?: string
          }>,
        ) => Promise<{
          ok: boolean
          steps: Array<{
            index: number
            actionKind: string
            status: string
            message: string
          }>
        }>
      }
      workflow: {
        runActions: (
          actions: Array<{
            kind: string
            selector?: string
            text?: string
            script?: string
            value?: string
          }>,
        ) => Promise<{
          ok: boolean
          steps: Array<{
            index: number
            actionKind: string
            status: string
            message: string
          }>
        }>
        sleep: (milliseconds: number) => Promise<void>
        log: (...args: unknown[]) => void
      }
      onRouteChange: (
        handler: (route: {
          href: string
          pathname: string
          search: string
          hash: string
        }) => void,
      ) => void
    }
  }
}
