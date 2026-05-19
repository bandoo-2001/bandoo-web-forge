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
