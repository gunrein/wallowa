import * as Plot from "@observablehq/plot";
import { tableFromIPC } from "@apache-arrow/ts";

async function doPlot(startDate: Date, endDate: Date) {
  const data = await tableFromIPC(fetch(`/data/github/merged_pr_duration_30_day_rolling_avg_hours.arrow?start_date=${startDate.toISOString()}&end_date=${endDate.toISOString()}`))
  const plot = Plot.plot({
      style: "overflow: visible;",
      y: {grid: true},
      marks: [
        Plot.ruleY([0]),
        Plot.lineY(data, {x: "day", y: "duration", stroke: "repo"}),
      ],
      color: { legend: true },
    })
  const div = document.querySelector("#vis")
  if (div) div.replaceChildren(plot)
}

function dateAtStartOfDayUTC(date: Date): Date {
  date.setUTCHours(0, 0, 0, 0);
  return date;
}

function dateOffsetUTC(date: Date, daysToOffset: number): Date {
  const offsetDate = new Date(date);
  offsetDate.setDate(date.getUTCDate() - daysToOffset);
  return offsetDate;
}

function justDatePartAsStringUTC(date: Date): string {
  return date.toISOString().split('T')[0];
}

function parseOffset(range: string): number {
  let offset = 30; // default to 30 days of offset
  switch (range) {
    case 'last_thirty':
      offset = 30;
      break;
    case 'last_seven':
      offset = 7;
      break;
    case 'last_ninety':
      offset = 90;
      break;
    case 'last_three_sixty_five':
      offset = 365;
      break;
    default:
      console.error(`Unexpected time range value ${range}`)
  }  
  return offset;
}

function updateAbsoluteRange(startDate: Date, endDate: Date) {
    const startDateEl = document.querySelector<HTMLInputElement>('#start_date');
    const endDateEl = document.querySelector<HTMLInputElement>('#end_date');

    if (endDateEl) {
      endDateEl.value = justDatePartAsStringUTC(endDate);
    }
    if (startDateEl) {
      startDateEl.value = justDatePartAsStringUTC(startDate);
    }  
}

function getAbsoluteRange(): { startDate: Date; endDate: Date } {
  let endDate: Date;
  const endDateStr = document.querySelector<HTMLInputElement>('#end_date')?.value;
  if (!endDateStr) {
    endDate = dateAtStartOfDayUTC(new Date());
  } else {
    endDate = new Date(endDateStr);
  }

  let startDate: Date;
  const startDateStr = document.querySelector<HTMLInputElement>('#start_date')?.value;
  if (!startDateStr) {
    startDate = dateOffsetUTC(endDate, 30);
  } else {
    startDate = new Date(startDateStr);
  }

  return { startDate, endDate };
}

function timeRangeChanged(ev: Event) {
  const range = (ev.target as HTMLInputElement)?.value;

  if (range === 'absolute') {
    document.querySelector("#absolute_range_inputs")?.classList.remove('hidden');
    const { startDate, endDate } = getAbsoluteRange();
    updateAbsoluteRange(startDate, endDate);
    doPlot(startDate, endDate);
  } else {
    document.querySelector("#absolute_range_inputs")?.classList.add('hidden');
    let offset = parseOffset(range);
    const endDate = dateAtStartOfDayUTC(new Date());
    const startDate = dateOffsetUTC(endDate, offset);
    updateAbsoluteRange(startDate, endDate);
    doPlot(startDate, endDate)
  }
}

document.querySelector("#time_range")?.addEventListener("input", timeRangeChanged);

const endDate = dateAtStartOfDayUTC(new Date());
const startDate = dateOffsetUTC(endDate, 30);
updateAbsoluteRange(startDate, endDate);
doPlot(startDate, endDate);