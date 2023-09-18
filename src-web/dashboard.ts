import * as Plot from "@observablehq/plot";
import { tableFromIPC } from "@apache-arrow/ts";

async function doPlot() {
  doPlotGitHubPRDuration();
  doPlotGitHubClosedPRCount();
}

async function doPlotGitHubPRDuration() {
  ({ range, startDate, endDate } = getDateRange());
  const url = new URL('/data/github/merged_pr_duration_rolling_daily_average.arrow', window.location.origin);
  url.searchParams.append('start_date', startDate.toISOString());
  url.searchParams.append('end_date', endDate.toISOString());

  const data = await tableFromIPC(fetch(url))
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
  const div = document.querySelector("#github_pr_duration")
  if (div) div.replaceChildren(plot)
}

async function doPlotGitHubClosedPRCount() {
  ({ range, startDate, endDate } = getDateRange());
  const url = new URL('/data/github/closed_prs.arrow', window.location.origin);
  url.searchParams.append('start_date', startDate.toISOString());
  url.searchParams.append('end_date', endDate.toISOString());

  const data = await tableFromIPC(fetch(url))
  // If the date range is larger than 10 weeks, group the data by week instead of day
  const dayDiff = Math.ceil(Math.abs((endDate.getTime() - startDate.getTime()) / (1000 * 60 * 60 * 24)))
  const xInterval = (dayDiff > (7 * 10)) ? "week" : "day"

  const plot = Plot.plot({
      style: "overflow: visible;",
      y: { grid: true },
      color: { legend: true },
      marks: [
        Plot.axisX({ label: "Date", interval: xInterval, ticks: 6 }),
        Plot.ruleY([0]),
        Plot.axisY({ label: `Count of closed PRs by ${xInterval}` }),
        // @ts-ignore
        Plot.rectY(data, Plot.binX({ y: "count" }, { x: "closed_at", interval: xInterval, fill: "repo", fx: "repo", tip: true })),
      ],
    })
  const div = document.querySelector("#github_closed_pr_count")
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

function getDateRange(): { range: string, startDate: Date; endDate: Date } {
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

  return { range, startDate, endDate }
}

function dateRangeChanged(_ev: Event) {
  ({ range, startDate, endDate } = getDateRange());
  localStorage.setItem('dateRange', JSON.stringify({ range, startDate, endDate }));
  updateAbsoluteRange(startDate, endDate);
  doPlot();
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

doPlot();