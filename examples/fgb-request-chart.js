// A visualization of the HTTP Range requests flatgeobuf makes to satisfy a
// bounding box query, and where they land in the file being read.
//
// Usage:
//
//     fgbRequestChart.watch(url, document.getElementById("requests"));
//     ...
//     fgbRequestChart.startQuery();          // before each query
//     fgbRequestChart.headerMeta(headerMeta) // from your headerMetaFn
//
window.fgbRequestChart = (() => {
    // Everything we've learned about the requests made for the most recent query.
    const state = {
        url: null,
        container: null,
        requests: [],
        fileSize: null,
        // byte length of the flatbuffers header, *excluding* the magic bytes
        // and the 4 byte length prefix which precede it
        headerLength: null,
        indexLength: null,
    };

    function lengthBeforeIndex() {
        // [magic bytes (8)][header length (4)][header]
        return 12 + state.headerLength;
    }

    function lengthBeforeData() {
        return lengthBeforeIndex() + state.indexLength;
    }

    // How much of the file we know about. Content-Range tells us the total
    // size, but only when the server is same-origin or exposes the header to
    // CORS - otherwise the best we can say is "at least this much".
    function fileExtent() {
        if (state.fileSize !== null) return state.fileSize;
        return state.requests.reduce((furthest, r) => Math.max(furthest, r.end + 1), 0);
    }

    // The same computation the reader uses to find the size of the index:
    // a packed R-tree of fixed size nodes.
    //
    // The index is optional - a file written without one has an indexNodeSize
    // of 0, and its data starts right after the header.
    //
    // REVIEW: this could be deduped by exporting calcTreeSize (and
    // NODE_ITEM_BYTE_LEN) from packedrtree.ts through the geojson and ol entry
    // points.
    function calcTreeSize(numItems, nodeSize) {
        const NODE_ITEM_BYTE_LEN = 40;
        if (nodeSize === 0 || numItems === 0) return 0;
        nodeSize = Math.min(Math.max(+nodeSize, 2), 65535);
        let n = numItems;
        let numNodes = n;
        do {
            n = Math.ceil(n / nodeSize);
            numNodes += n;
        } while (n !== 1);
        return numNodes * NODE_ITEM_BYTE_LEN;
    }

    // The sections of the file, in order. A file written without a spatial
    // index has no index section at all, so we drop any empty section.
    function sections() {
        const knownSize = state.fileSize !== null;
        return [
            // The header is a couple of kilobytes: too small to be worth a row
            // of its own, and it still shows up as a sliver on the rows below.
            { region: 'header', start: 0, end: lengthBeforeIndex(), ownRow: false },
            { region: 'index', start: lengthBeforeIndex(), end: lengthBeforeData() },
            {
                region: 'data',
                start: lengthBeforeData(),
                end: fileExtent(),
                // we only know where the data ends if we know how big the file is
                approximate: !knownSize,
            },
        ].filter((section) => section.end > section.start);
    }

    // The sections that get a row of the chart to themselves.
    function rows() {
        return sections().filter((section) => section.ownRow !== false);
    }

    function fmtBytes(bytes) {
        if (bytes < 1024) return `${bytes} B`;
        if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
        if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
        return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
    }

    function sectionSize(section) {
        const size = fmtBytes(section.end - section.start);
        return section.approximate ? `≥ ${size}` : size;
    }

    // Sizes we could only bound get a marker, whose tooltip explains why.
    const APPROX_MARK = '*';
    const APPROX_NOTE =
        'We can only put a lower bound on this size. The total comes from the ' +
        'Content-Range header on the Range responses, which is not CORS safelisted - ' +
        'so for a file on another origin the server has to opt in with ' +
        'Access-Control-Expose-Headers: Content-Range before the browser will let ' +
        'this page read it. This one does not, so the most we can say is that the ' +
        'file reaches at least as far as the furthest byte we have fetched.';

    // How much of a section we pulled down the wire, as a share of the whole.
    // Where we only know a lower bound for the section's size a percentage
    // would be misleading, so we give the bytes instead.
    function fetchedShare(section) {
        const fetched = fetchedInSection(section);
        if (section.approximate) return `fetched ${fmtBytes(fetched)} of ${sectionSize(section)}`;

        const percent = (100 * fetched) / (section.end - section.start);
        let rounded;
        if (percent >= 10) rounded = percent.toFixed(0);
        else if (percent >= 1) rounded = percent.toFixed(1);
        else if (percent >= 0.01) rounded = percent.toFixed(2);
        else rounded = '<0.01';
        return `fetched ${rounded}% of ${sectionSize(section)}`;
    }

    // How many of a section's bytes we actually pulled down the wire.
    function fetchedInSection(section) {
        return state.requests.reduce((sum, request) => {
            const overlap = Math.min(request.end + 1, section.end) - Math.max(request.start, section.start);
            return sum + Math.max(0, overlap);
        }, 0);
    }

    // How much of a request was needed right now, vs. fetched speculatively?
    //
    // The reader deliberately overfetches: when the extra bytes are cheaper
    // than an extra round trip, it grabs them, hoping to serve a later read
    // from the buffer without another request.
    function classifyRequest(request) {
        if (request.start < lengthBeforeIndex()) {
            // Fetching the header, the reader optimistically grabs the first
            // few layers of the index along with it.
            const used = Math.min(lengthBeforeIndex() - request.start, request.length);
            return { region: 'header', used, overfetch: request.length - used };
        }
        if (request.start < lengthBeforeData()) {
            return { region: 'index', used: request.length, overfetch: 0 };
        }
        // A data request covers a batch of matching features. It may also span
        // over features we didn't ask for, when skipping them would have cost
        // an extra request - we can't tell those apart from out here.
        return { region: 'data', used: request.length, overfetch: 0 };
    }

    const LANE_HEIGHT = 16;
    const BAR_HEIGHT = 34;
    const CALLOUT_HEIGHT = 15;
    // room beside the bar for the arrows pointing back at it
    const ARROW_GAP = 12;
    const ARROW_HALF_WIDTH = 3;
    // breathing room above and below the request labels, which keeps them
    // roughly centred between the bar above and the bar below
    const REQUEST_BAND_PAD = 16;
    // how far into its span a request's label and leader sit
    const REQUEST_ANCHOR = 0.05;
    // Requests that land in the header or the index all reach the same few
    // pixels of the bar below, so each leader gets its own horizontal corridor
    // to run along before dropping, and drops a hair to the side of the last.
    // They take their corridors high in the band, right under the labels,
    // leaving the ones nearest the bottom bar to the requests that only appear
    // there.
    const CORRIDOR_GAP = 6;
    const CORRIDOR_STEP = 4;
    const CORRIDOR_SIDESTEP = 2;
    // a leader this close to straight is drawn straight rather than jogged
    const STRAIGHT_ENOUGH = 8;
    // the last stretch of a sidestepped leader, angling back onto its span
    const LANDING = 8;

    // roughly how wide a label will be, for packing purposes
    function labelWidth(text) {
        return text.length * 6.2;
    }

    // Where a row's bar sits, and how to place a byte offset along it. Each row
    // runs from byte 0 to the end of its own section, so it's a zoomed out view
    // of the row above it, with the earlier sections squeezed into a sliver on
    // the left.
    function layOutRow(section, width) {
        const rowEnd = section.end;
        return {
            section,
            // filled in once we know how tall the request band has to be
            barTop: 0,
            barBottom: BAR_HEIGHT,
            visible: sections().filter((other) => other.start < rowEnd),
            holds: (request) => request.start < rowEnd,
            clips: (request) => request.end + 1 > rowEnd,
            x: (byteOffset) => (Math.max(0, Math.min(byteOffset, rowEnd)) / rowEnd) * width,
        };
    }

    // The pale bar, with the stretches we actually pulled down the wire painted
    // over in full strength - so the bar shows how much of the file was read.
    function barSvg(row) {
        return row.visible
            .map((other) => {
                const left = row.x(other.start);
                const segmentWidth = row.x(other.end) - left;
                const readSvg = state.requests
                    .filter((request) => request.end >= other.start && request.start < other.end)
                    .map((request) => {
                        const readLeft = row.x(Math.max(request.start, other.start));
                        const readWidth = Math.max(1, row.x(Math.min(request.end + 1, other.end)) - readLeft);
                        return `<rect class="fgb-region-read ${other.region}" x="${readLeft}" y="${row.barTop}"
                                      width="${readWidth}" height="${BAR_HEIGHT}" />`;
                    })
                    .join('');

                return `<rect class="fgb-region ${other.region}" x="${left}" y="${row.barTop}"
                               width="${segmentWidth}" height="${BAR_HEIGHT}"><title>${other.region}: ${sectionSize(other)}</title></rect>
                        ${readSvg}`;
            })
            .join('');
    }

    // Names each of a row's sections off to the left of itself, stacked one per
    // line, with an arrow pointing back at the section it names. `side` says
    // whether the stack goes above the bar or below it.
    function calloutsSvg(row, width, side) {
        const above = side === 'above';
        const tipY = above ? row.barTop - 1 : row.barBottom + 1;

        const parts = row.visible.map((other, i) => {
            const caption = `${other.region} (${fetchedShare(other)})`;
            // hovering anywhere on the label - not just the marker - shows the
            // browser's own tooltip with the note
            const note = other.approximate ? `<title>${APPROX_NOTE}</title>` : '';
            const marker = other.approximate ? `<tspan class="fgb-approx-marker">${APPROX_MARK}</tspan>` : '';
            // The last section takes the line closest to the bar, and earlier
            // ones stack away from it. Their arrows land further left the
            // earlier they are, so this keeps a long arrow clear of the labels
            // it passes, which all start to the right of it.
            const line = row.visible.length - 1 - i;
            const y = above
                ? row.barTop - ARROW_GAP - line * CALLOUT_HEIGHT - 3
                : row.barBottom + ARROW_GAP + (line + 1) * CALLOUT_HEIGHT - 4;

            // the arrow lands a little way into the section it names, nudged
            // inward where a sliver would otherwise have its head clipped by
            // the edge of the drawing
            const left = row.x(other.start);
            const justInside = left + (row.x(other.end) - left) * 0.005;
            const tipX = Math.min(Math.max(justInside, ARROW_HALF_WIDTH), width - ARROW_HALF_WIDTH);
            // keep the label on the page if its section hugs the right edge
            const labelX = Math.min(tipX, width - labelWidth(caption));
            const tailY = above ? y + 3 : y - 9;

            // the head points along the arrow, which is upright when the label
            // sits directly above or below its section
            const angle = (Math.atan2(tipX - labelX, tailY - tipY) * 180) / Math.PI;
            return {
                // an arrow may pass a label on the line beyond it, so all the
                // text is drawn last, on top
                marks: `
                    <path class="fgb-callout-leader" d="M ${labelX} ${tailY} L ${tipX} ${tipY}" />
                    <path class="fgb-callout-arrow"
                          d="M 0 0 L -${ARROW_HALF_WIDTH} 5 L ${ARROW_HALF_WIDTH} 5 Z"
                          transform="translate(${tipX}, ${tipY}) rotate(${angle.toFixed(1)})" />
                `,
                text: `<text class="fgb-callout" x="${labelX}" y="${y}">${note}${caption}${marker}</text>`,
            };
        });

        return parts.map((part) => part.marks).join('') + parts.map((part) => part.text).join('');
    }

    // A request usually lands on more than one row - the header fetch shows up
    // on every row below it - so it gets one label, in the band between the
    // bars, with a line reaching out to its span on each bar it appears on.
    function placeRequests(rowLayouts, width) {
        const laneEnds = [];
        // Labels read in request order, left to right. Each bar has its own
        // scale, so a request anchored on the lower bar can otherwise come out
        // left of an earlier one anchored on the upper bar.
        let leftmost = 0;
        return state.requests
            .map((request) => {
                const spans = rowLayouts
                    .filter((row) => row.holds(request))
                    .map((row) => {
                        const left = row.x(request.start);
                        const spanWidth = Math.max(2, row.x(request.end + 1) - left);
                        return { row, left, spanWidth, anchorX: left + spanWidth * REQUEST_ANCHOR };
                    });
                return { request, spans };
            })
            .filter(({ spans }) => spans.length > 0)
            .map(({ request, spans }) => {
                const { overfetch } = classifyRequest(request);
                // the size includes any overfetch; the summary totals that up
                const label = `#${request.number} · ${fmtBytes(request.length)}`;

                // The label is centred over the request's span on the topmost
                // bar it reaches, so its leader drops straight out of the middle
                // of the text - but it is never left of the label before it, nor
                // so far over that it runs off either edge.
                const half = labelWidth(label) / 2;
                const wanted = Math.max(spans[0].anchorX - half, leftmost);
                const labelLeft = Math.max(0, Math.min(wanted, width - labelWidth(label)));
                leftmost = labelLeft + 4;

                let lane = laneEnds.findIndex((end) => end <= labelLeft);
                if (lane === -1) lane = laneEnds.length;
                laneEnds[lane] = labelLeft + labelWidth(label) + 12;

                // whether it also appears on the topmost bar, which decides
                // where its leader runs across
                const onTopBar = spans[0].row === rowLayouts[0];

                return { request, overfetch, label, spans, labelLeft, lane, onTopBar };
            });
    }

    function requestsSvg(placed, band, width) {
        let upper = 0;
        let lower = 0;
        const parts = placed.map(({ request, overfetch, label, spans, labelLeft, lane, onTopBar }) => {
            const labelY = band.top + band.pad + (lane + 1) * LANE_HEIGHT - 4;
            const title = [
                `request #${request.number}: ${fmtBytes(request.length)}`,
                `bytes ${request.start.toLocaleString()}-${request.end.toLocaleString()}`,
                overfetch > 0 ? `overfetch: ${fmtBytes(overfetch)}` : null,
                request.durationMs !== null ? `${request.durationMs.toFixed(0)} ms` : 'in flight',
            ]
                .filter((part) => part !== null)
                .join('\n');

            const marks = spans
                .map(({ row, left, spanWidth, anchorX }) => {
                    // this request runs off the end of this row, and picks up
                    // again on the row below
                    const clipped = row.clips(request) ? ' clipped' : '';
                    const outline = `<rect class="fgb-request-outline${clipped}" x="${left}" y="${row.barTop}"
                                           width="${spanWidth}" height="${BAR_HEIGHT}"><title>${title}</title></rect>`;

                    // leaders set off from the middle of the label's text
                    const fromX = labelLeft + labelWidth(label) / 2;

                    // a bar above the band is reached straight from the label
                    if (row.barBottom <= band.top) {
                        const up =
                            Math.abs(fromX - anchorX) <= STRAIGHT_ENOUGH
                                ? `M ${anchorX} ${labelY - 9} V ${row.barBottom}`
                                : `M ${fromX} ${labelY - 9} H ${anchorX} V ${row.barBottom}`;
                        return `
                            <path class="fgb-callout-leader" d="${up}" />
                            ${outline}
                        `;
                    }

                    // A bar below the band is reached by way of this leader's
                    // own corridor. A request that also appears on the bar above
                    // runs across just under the labels and then drops the whole
                    // way, since its span down here is a sliver alongside every
                    // other one of those; the rest, whose spans are spread out,
                    // run across just above the bar they are dropping into.
                    let corridorY;
                    let dropX;
                    if (onTopBar) {
                        corridorY = band.labelsBottom + CORRIDOR_GAP + upper * CORRIDOR_STEP;
                        dropX = Math.min(anchorX + upper * CORRIDOR_SIDESTEP, width - 1);
                        upper += 1;
                    } else {
                        corridorY = row.barTop - CORRIDOR_GAP - lower * CORRIDOR_STEP;
                        dropX = anchorX;
                        lower += 1;
                    }
                    // a leader that stepped aside to keep clear of its
                    // neighbours angles back so that it still lands on its span
                    const landing =
                        dropX === anchorX ? `V ${row.barTop}` : `V ${row.barTop - LANDING} L ${anchorX} ${row.barTop}`;
                    const down =
                        Math.abs(fromX - dropX) <= STRAIGHT_ENOUGH && dropX === anchorX
                            ? `M ${dropX} ${labelY + 3} V ${row.barTop}`
                            : `M ${fromX} ${labelY + 3} V ${corridorY} H ${dropX} ${landing}`;
                    return `
                        <path class="fgb-callout-leader" d="${down}" />
                        ${outline}
                    `;
                })
                .join('');

            return {
                // a leader may cross a neighbouring label, so all the text is
                // drawn last, on top
                marks,
                text: `<text class="fgb-request-label" x="${labelLeft}" y="${labelY}">${label}</text>`,
            };
        });

        return parts.map((part) => part.marks).join('') + parts.map((part) => part.text).join('');
    }

    // Draws the whole chart as a single drawing, so that a request's label can
    // reach across from one bar to the next.
    function renderChart(chart) {
        const width = chart.clientWidth || 800;
        const rowLayouts = rows().map((section) => layOutRow(section, width));

        // Lane packing only needs the horizontal placement, so we can work out
        // how tall the request band is before we know where anything sits.
        const placed = placeRequests(rowLayouts, width);
        const lanes = placed.reduce((most, span) => Math.max(most, span.lane + 1), 1);

        // The band has to be deep enough for both runs of corridors - the one
        // under the labels and the one above the lower bar - and we leave as
        // much room above the labels as below, so they stay centred.
        const descending = placed.filter((span) => span.spans.some((s) => s.row !== rowLayouts[0]));
        const viaUpper = descending.filter((span) => span.onTopBar).length;
        const viaLower = descending.length - viaUpper;
        const corridors = 2 * CORRIDOR_GAP + (viaUpper + viaLower) * CORRIDOR_STEP + 4;
        const bandPad = Math.max(REQUEST_BAND_PAD, corridors);
        const bandHeight = lanes * LANE_HEIGHT + 2 * bandPad;

        // The topmost bar's sections are named above it and every other bar's
        // below, which leaves the band between the bars - or below a lone bar -
        // free for the request labels.
        let y = rowLayouts[0].visible.length * CALLOUT_HEIGHT + ARROW_GAP;
        let bandTop = 0;
        rowLayouts.forEach((row, i) => {
            row.barTop = y;
            row.barBottom = y + BAR_HEIGHT;
            y = row.barBottom;
            if (i === 0) {
                bandTop = y;
                y += bandHeight;
            } else {
                y += ARROW_GAP + row.visible.length * CALLOUT_HEIGHT;
            }
        });
        const height = y;

        const band = { top: bandTop, pad: bandPad, labelsBottom: bandTop + bandPad + lanes * LANE_HEIGHT };

        const bars = rowLayouts.map((row) => barSvg(row)).join('');
        const names = rowLayouts.map((row, i) => calloutsSvg(row, width, i === 0 ? 'above' : 'below')).join('');

        chart.innerHTML = `
            <svg width="${width}" height="${height}" viewBox="0 0 ${width} ${height}">
                ${bars}
                ${names}
                ${requestsSvg(placed, band, width)}
            </svg>
        `;
    }

    function render() {
        if (!state.container) return;
        const chart = state.container.querySelector('.fgb-request-chart');
        // we can't place anything until we know where the sections begin and end
        if (state.headerLength === null || state.indexLength === null || fileExtent() === 0) return;

        renderChart(chart);

        const count = state.requests.length;
        const fetched = state.requests.reduce((sum, r) => sum + r.length, 0);
        const share = state.fileSize !== null ? ` (${((100 * fetched) / state.fileSize).toFixed(1)}% of the file)` : '';
        const fileSize = sectionSize({ start: 0, end: fileExtent(), approximate: state.fileSize === null });
        const marker =
            state.fileSize === null
                ? `<abbr class="fgb-approx-marker" title="${APPROX_NOTE}">${APPROX_MARK}</abbr>`
                : '';
        state.container.querySelector('.fgb-request-summary').innerHTML =
            `<strong>${count} request${count === 1 ? '' : 's'}</strong>, ` +
            `fetched ${fmtBytes(fetched)} of ${fileSize}${marker}${share}`;
    }

    function throttle(fn, ms) {
        let timer = null;
        return () => {
            if (timer !== null) return;
            timer = setTimeout(() => {
                timer = null;
                fn();
            }, ms);
        };
    }

    window.addEventListener('resize', throttle(render, 200));

    // Wrap fetch so we can log the Range requests flatgeobuf makes for us.
    function interceptRequests() {
        const nativeFetch = window.fetch.bind(window);
        window.fetch = async (input, init) => {
            const url = typeof input === 'string' ? input : input.url;
            const range = new Headers(init?.headers || {}).get('Range');
            const match = range && /bytes=(\d+)-(\d+)/.exec(range);
            if (url !== state.url || !match) {
                return nativeFetch(input, init);
            }

            const start = Number(match[1]);
            const end = Number(match[2]);
            const request = {
                number: state.requests.length + 1,
                start,
                end,
                length: end - start + 1,
                startedAt: performance.now(),
                durationMs: null,
            };
            state.requests.push(request);

            const response = await nativeFetch(input, init);
            request.durationMs = performance.now() - request.startedAt;

            // e.g. "bytes 0-12943/14100008". Only readable when the server is
            // same-origin or lists Content-Range in Access-Control-Expose-Headers.
            const contentRange = response.headers.get('Content-Range');
            const total = contentRange && /\/(\d+)/.exec(contentRange);
            if (total) state.fileSize = Number(total[1]);

            // The very first request starts at byte 0, so it tells us how long
            // the header is - and therefore where the index starts. Anything
            // shorter than [magic bytes][header length] can't, and we mustn't
            // throw out here: we're standing in the middle of someone's fetch.
            if (start === 0) {
                const bytes = await response.clone().arrayBuffer();
                if (bytes.byteLength >= 12) {
                    state.headerLength = new DataView(bytes).getUint32(8, true);
                }
            }

            render();
            return response;
        };
    }

    return {
        // Start logging requests for `url`, and draw them into `container`.
        watch(url, container) {
            state.url = url;
            state.container = container;
            const filename = url.split('/').pop().split('?')[0];
            container.innerHTML = `
                <h3>Range Requests for ${filename}</h3>
                <p class="fgb-request-summary"></p>
                <div class="fgb-request-chart"></div>
                <p>
                    A .fgb file is laid out as <em>[header][optional index][data]</em>.
                    The index and the data each get a row above, drawn pale, with the
                    stretches fetched for the query on the map painted in.
                </p>
                <p>
                    Every row starts at byte 0 but is drawn at its own scale, so the
                    sections before it collapse into a sliver on the left - the header
                    is a couple of kilobytes, too small to see at these scales. A
                    request's size includes any overfetch: bytes the reader took
                    speculatively, because they were cheaper than another round trip.
                </p>
            `;
            interceptRequests();
        },

        // Forget the previous query's requests: each query re-opens the file.
        startQuery() {
            state.requests = [];
        },

        // Learn where the index ends and the data begins.
        headerMeta(headerMeta) {
            state.indexLength = calcTreeSize(Number(headerMeta.featuresCount), Number(headerMeta.indexNodeSize));
            render();
        },
    };
})();
