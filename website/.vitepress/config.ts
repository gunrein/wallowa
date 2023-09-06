import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Wallowa",
  description: "Measure your Software Development Life Cycle (SDLC)",
  lang: 'en-US',
  head: [
    ['link', { rel: 'icon', href: '/wallowa-logo.svg' }],
    ['link', { rel: 'apple-touch-icon', href: '/apple-touch-icon.png' }],
    ['link', { rel: 'apple-touch-icon', sizes: "152x152", href: '/touch-icon-ipad.png' }],
    ['link', { rel: 'apple-touch-icon', sizes: "180x180", href: '/touch-icon-iphone-retina' }],
    ['link', { rel: 'apple-touch-icon', sizes: "167x167", href: '/touch-icon-ipad-retina.png' }],
  ],
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    logo: '/wallowa-logo.svg',

    search: {
      provider: 'local'
    },
    
    nav: [
      { text: 'Docs', link: '/docs/'},
      { text: 'Changelog', link: '/docs/changelog'},
    ],

    sidebar: {
      '/docs/': [{
        text: 'Documentation',
        items: [
          { text: 'Introduction', link: '/docs/' },
          { 
            text: "Guides",
            collapsed: false,
            items: [
              { text: 'Get started', link: '/docs/get-started' },
              { text: 'Data analysis', link: '/docs/data-analysis' },
              { text: 'Contributing', link: '/docs/contributing' },
            ],
          },
          { 
            text: "Reference",
            collapsed: false,
            items: [
              { 
                text: 'Data sources', link: '/docs/sources/',
                collapsed: false,
                items: [
                  { text: 'GitHub', link: '/docs/sources/github' }
                ]
              },
              { text: 'CLI', link: '/docs/cli' },
              { text: 'Web UI', link: '/docs/web-ui' },
              { text: 'Configuration', link: '/docs/configuration' },
              { text: 'Changelog', link: '/docs/changelog' },    
            ] 
          },
          { 
            text: "Explanation",
            collapsed: false,
            items: [
              { text: 'Architecture', link: '/docs/architecture' },
            ]
          },
        ]
      }]
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/gunrein/wallowa' }
    ],

    footer: {
      message: 'Released under the <a href="https://github.com/gunrein/wallowa/blob/main/LICENSE">MIT License</a>.',
      copyright: 'Copyright © 2023-present <a href="https://github.com/gunrein">Greg Unrein</a>'
    },

    editLink: {
      pattern: 'https://github.com/gunrein/wallowa/edit/main/website/:path'
    }
  }
})
