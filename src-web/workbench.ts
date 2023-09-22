import {LitElement, html} from 'lit';
import {customElement} from 'lit/decorators.js';

@customElement('wd-text')
class WdText extends LitElement {
  render() {
    return html`
      <div>Hello from MyElement!</div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wd-text": WdText;
  }  
}