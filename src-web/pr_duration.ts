import * as Plot from "@observablehq/plot";
import { tableFromIPC } from "@apache-arrow/ts";

async function doPlot(startDate: Date, endDate: Date) {
  const data = await tableFromIPC(fetch(`/data/github/merged_pr_duration_rolling_daily_average.arrow?start_date=${startDate.toISOString()}&end_date=${endDate.toISOString()}`))
  const plot = Plot.plot({
      style: "overflow: visible;",
      y: {grid: true},
      marks: [
        Plot.axisX({label: "Date" }),
        Plot.ruleY([0]),
        Plot.axisY({label: "Rolling 30-day average number of days to merge"}),
        Plot.lineY(data, {x: "day", y: "duration", stroke: "repo", tip: "x"}),
        Plot.crosshairX(data, {x: "day", y: "duration"})
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

function dateRangeChanged(_ev: Event) {
  const range = document.querySelector<HTMLInputElement>("#date_range")?.value ?? 'last_thirty';

  let startDate: Date, endDate: Date;
  if (range === 'absolute') {
    document.querySelector("#absolute_range_inputs")?.classList.remove('hidden');
    ({ startDate, endDate } = getAbsoluteRange());
  } else {
    document.querySelector("#absolute_range_inputs")?.classList.add('hidden');
    endDate = dateAtStartOfDayUTC(new Date());
    startDate = dateOffsetUTC(endDate, parseOffset(range));
  }
  localStorage.setItem('dateRange', JSON.stringify({ range, startDate, endDate }));
  updateAbsoluteRange(startDate, endDate);
  doPlot(startDate, endDate);
}

// Setup the default date range and load any stored date range information
let endDate = dateAtStartOfDayUTC(new Date());
let startDate = dateOffsetUTC(endDate, 30);
let range = 'last_thirty';
const storedDateRange = localStorage.getItem('dateRange');
if (storedDateRange) {
  ({ range, startDate, endDate } = JSON.parse(storedDateRange));
  // When the range isn't absolute then the endDate needs to be today (UTC) and the startDate needs
  // to be updated relative to endDate instead of the stored values being used. Otherwise the 
  // date range used will be incorrect, but hard to spot by the user.
  if (range != 'absolute') {
    endDate = dateAtStartOfDayUTC(new Date());
    startDate = dateOffsetUTC(endDate, parseOffset(range));
  } else {
    document.querySelector("#absolute_range_inputs")?.classList.remove('hidden');
    // Since the stored range is absolute, update both startDate and endDate with the stored
    // date values
    startDate = new Date(startDate);
    endDate = new Date(endDate);
  }
  const dateRangeEl = document.querySelector<HTMLInputElement>("#date_range");
  if (dateRangeEl) {
    dateRangeEl.value = range;
  }
}
updateAbsoluteRange(startDate, endDate);

document.querySelector("#date_range")?.addEventListener("input", dateRangeChanged);
document.querySelector("#start_date")?.addEventListener("input", dateRangeChanged);
document.querySelector("#end_date")?.addEventListener("input", dateRangeChanged);

doPlot(startDate, endDate);