// https://vitepress.dev/guide/custom-theme
import { h } from 'vue'
import Theme from 'vitepress/theme'
import './style.css'

function ConvertKitComponent() {
  return h(
    'div',
    {
      class: 'convertkit',
    },
    [
      h(
        'script',
        {
          'async': '',
          'data-uid': '55c0188c0f',
          'src': 'https://wallowa.ck.page/55c0188c0f/index.js',
        },
      ),        
    ]
  )
}

export default {
  extends: Theme,
  Layout: () => {
    return h(Theme.Layout, null, {
      // https://vitepress.dev/guide/extending-default-theme#layout-slots
      'home-features-after': () => h(ConvertKitComponent)
    })
  },
  enhanceApp({ app, router, siteData }) {
    // ...
  }
}
