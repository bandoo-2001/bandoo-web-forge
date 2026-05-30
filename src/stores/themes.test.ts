import { describe, expect, it } from 'vitest'
import { defaultAppSettings, defaultChromeConfig } from './webapps'
import { mergeChromeConfig } from './themes'

describe('mergeChromeConfig', () => {
  it('applies global defaults, preset values, and app overrides in order', () => {
    const settings = defaultAppSettings()
    settings.defaultChromeConfig = {
      ...defaultChromeConfig(),
      backgroundColor: '#111111',
      foregroundColor: '#eeeeee',
    }

    const chrome = mergeChromeConfig(
      {
        themePresetId: 'custom',
        cornerRadius: 22,
      },
      settings,
      [
        {
          id: 'custom',
          name: 'Custom',
          chromeConfig: {
            ...defaultChromeConfig(),
            backgroundColor: '#334455',
            controlsPosition: 'left',
          },
          createdAt: 1,
        },
      ],
    )

    expect(chrome.backgroundColor).toBe('#334455')
    expect(chrome.foregroundColor).toBe('#f8fafc')
    expect(chrome.controlsPosition).toBe('left')
    expect(chrome.cornerRadius).toBe(22)
  })
})
