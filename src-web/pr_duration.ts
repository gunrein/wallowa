import * as Plot from "@observablehq/plot";
import { tableFromIPC } from "@apache-arrow/ts";

async function doPlot() {
    const data = await tableFromIPC(fetch("/query/merged_pr_duration_30_day_rolling_avg_hours.arrow"));
    const plot = Plot.plot({
        style: "overflow: visible;",
        y: {grid: true},
        marks: [
          Plot.ruleY([0]),
          Plot.lineY(data, {x: "day", y: "duration", stroke: "repo"}),
          Plot.text(data, Plot.selectLast({x: "day", y: "duration", z: "repo", text: "repo", textAnchor: "start", dx: 3}))
        ]
      })
    const div = document.querySelector("#vis");
    if (div) div.append(plot);
}

doPlot()