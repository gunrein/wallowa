import * as Plot from "@observablehq/plot";
import { tableFromIPC } from "@apache-arrow/ts";

async function doPlot() {
    const data = await tableFromIPC(fetch("/data/github/merged_pr_duration_30_day_rolling_avg_hours.arrow"));
    const plot = Plot.plot({
        style: "overflow: visible;",
        y: {grid: true},
        marks: [
          Plot.ruleY([0]),
          Plot.lineY(data, {x: "day", y: "duration", stroke: "repo"}),
        ],
        color: { legend: true },
      })
    const div = document.querySelector("#vis");
    if (div) div.append(plot);
}

doPlot()