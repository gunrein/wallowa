import {LitElement} from 'lit';
import {customElement} from 'lit/decorators.js';
import * as Plot from "@observablehq/plot";


@customElement('wd-plot')
class WdPlot extends LitElement {
  render() {
    const plot = Plot.rectY({length: 10000}, Plot.binX({y: "count"}, {x: Math.random})).plot();
    return plot;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wd-plot": WdPlot;
  }  
}