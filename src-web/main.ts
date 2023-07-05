import htmx from 'htmx.org';
import Alpine from 'alpinejs';

declare global {
  interface Window {
      Alpine: typeof Alpine;
      htmx: typeof htmx;
  }
}

window.htmx = htmx;

// Alternative to `import '../node_modules/htmx.org/dist/ext/disable-element.js';` that works
// - having trouble with globally defining `htmx` for the extension to reference
htmx.defineExtension('disable-element', {
  onEvent: function (name, evt) {
      let elt = evt.detail.elt;
      let target = elt.getAttribute("hx-disable-element");
      let targetElement = (target == "self") ? elt : document.querySelector(target);

      if (name === "htmx:beforeRequest" && targetElement) {
          targetElement.disabled = true;
      } else if (name == "htmx:afterRequest" && targetElement) {
          targetElement.disabled = false;
      }
  }
});

window.Alpine = Alpine;
Alpine.start();

/* TODO move this to use Alpinejs
htmx.onLoad(function(elt) {
  // Navigation menu toggle
  htmx.find('#navbar-toggle').addEventListener('change', function(ev) {
    elt.findAll.querySelectorAll('.menu-item-hidden-small').forEach((node) => {
      node.classList.toggle('hidden');
    });
  });
});
*/