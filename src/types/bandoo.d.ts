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
      clipboard: {
        readText: () => Promise<string>
        writeText: (text: string) => Promise<boolean>
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
        ) => Promise<{ ok: boolean; steps: number }>
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
