import "/static/htmx-1.9.2.min.js";
import "/static/htmx-ext-disable-element-1.9.2.js";
import "/static/alpinejs-3.12.1.min.js";

htmx.onLoad(function(elt) {
  // Navigation menu toggle
  htmx.find('#navbar-toggle').addEventListener('change', function(ev) {
    elt.findAll.querySelectorAll('.menu-item-hidden-small').forEach((node) => {
      node.classList.toggle('hidden');
    });
  });
});
