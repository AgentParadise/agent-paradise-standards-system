//! VSA (Vertical Slice Architecture) Visualization
//!
//! Matrix showing the intersection of feature slices and architectural layers.
//! - **Columns** = feature slices
//! - **Rows** = architectural layers
//! - **Cells** = module count with health indicator

use crate::escape_json_for_html;

/// Generate VSA Diagram HTML visualization.
///
/// # Arguments
/// * `modules_json` - JSON array of module data with slice and layer fields
///
/// # Returns
/// Complete HTML document as a string
#[allow(clippy::uninlined_format_args)]
pub fn generate(modules_json: &str) -> String {
    let modules_escaped = escape_json_for_html(modules_json);

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>VSA Diagram - Topology Visualization</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, sans-serif; background: #0a0a0f; color: #fff; padding: 20px; }}
        h1 {{ color: #00ff88; margin-bottom: 10px; }}
        .subtitle {{ color: #666; margin-bottom: 30px; }}
        .matrix-container {{ overflow-x: auto; }}
        table {{ border-collapse: collapse; min-width: 100%; }}
        th, td {{ padding: 12px 16px; text-align: center; border: 1px solid #222; min-width: 140px; }}
        th {{ background: #1a1a20; color: #888; font-weight: 500; position: sticky; top: 0; }}
        th.layer-header {{ writing-mode: horizontal-tb; background: #15151a; }}
        .layer-label {{ background: #15151a; font-weight: 500; text-align: left; color: #888; }}
        .cell {{ position: relative; cursor: pointer; transition: transform 0.2s; vertical-align: top; }}
        .cell:hover {{ transform: scale(1.02); z-index: 10; }}
        .cell-inner {{ border-radius: 6px; padding: 8px; min-height: 50px; display: flex; flex-direction: column; justify-content: center; align-items: center; }}
        .cell-inner.has-features {{ align-items: flex-start; gap: 6px; }}
        .cell-header {{ display: flex; justify-content: space-between; width: 100%; align-items: baseline; }}
        .cell-count {{ font-size: 20px; font-weight: 600; }}
        .cell-label {{ font-size: 10px; color: rgba(255,255,255,0.6); margin-top: 4px; }}
        .feature-tags {{ display: flex; flex-wrap: wrap; gap: 3px; width: 100%; }}
        .feature-tag {{ font-size: 9px; padding: 2px 6px; border-radius: 3px; background: rgba(255,255,255,0.08); color: rgba(255,255,255,0.7); white-space: nowrap; }}
        .empty {{ background: #0f0f12; color: #333; }}
        .legend {{ margin-top: 30px; display: flex; gap: 20px; flex-wrap: wrap; }}
        .legend-item {{ display: flex; align-items: center; gap: 8px; font-size: 12px; color: #888; }}
        .legend-color {{ width: 20px; height: 20px; border-radius: 4px; }}
        #detail-panel {{ position: fixed; top: 20px; right: 20px; width: 340px; max-height: calc(100vh - 40px); background: #12121a; border: 1px solid #333; border-radius: 10px; padding: 16px; font-size: 12px; z-index: 200; overflow-y: auto; display: none; }}
        #detail-panel.visible {{ display: block; }}
        #detail-panel h3 {{ color: #00ff88; margin-bottom: 10px; font-size: 14px; }}
        #detail-panel .close-btn {{ position: absolute; top: 10px; right: 14px; color: #666; cursor: pointer; font-size: 18px; line-height: 1; }}
        #detail-panel .close-btn:hover {{ color: #fff; }}
        #detail-panel .module-list {{ }}
        #detail-panel .module-item {{ padding: 3px 0; border-bottom: 1px solid #1a1a22; color: #ccc; }}
        #detail-panel .module-item .mod-name {{ color: #fff; }}
        #detail-panel .feature-group {{ margin-top: 8px; }}
        #detail-panel .feature-group-header {{ color: #00bbff; font-weight: 500; margin-bottom: 3px; font-size: 13px; }}
        #detail-panel .summary {{ color: #888; margin-bottom: 12px; font-size: 11px; }}
        .cell.selected .cell-inner {{ outline: 2px solid #00ff88; outline-offset: -2px; }}
        body.has-panel .matrix-container {{ margin-right: 360px; }}
    </style>
</head>
<body>
    <h1>🍰 Vertical Slice Architecture</h1>
    <p class="subtitle">Columns = bounded contexts, Rows = architectural layers — click a cell to inspect</p>

    <div class="matrix-container">
        <table id="matrix"></table>
    </div>

    <div class="legend">
        <div class="legend-item"><div class="legend-color" style="background:#00ff88"></div>Excellent health</div>
        <div class="legend-item"><div class="legend-color" style="background:#88cc55"></div>OK health</div>
        <div class="legend-item"><div class="legend-color" style="background:#ff7744"></div>Poor health</div>
        <div class="legend-item"><div class="legend-color" style="background:#0f0f12;border:1px solid #333"></div>Empty (no modules)</div>
    </div>

    <div id="detail-panel"><span class="close-btn">&times;</span><div id="detail-content"></div></div>

    <script>
        const MODULES = {modules_json};
        // Architectural priority: slices and aggregates first (core VSA),
        // then domain subdivisions, then infrastructure, then misc at bottom.
        const LAYER_ORDER = [
            'slices', 'aggregates',
            'commands', 'queries', 'events', 'read_models', 'services',
            'ports', '_shared',
        ];
        const allLayers = [...new Set(MODULES.map(m => m.layer))];
        const LAYERS = [
            ...LAYER_ORDER.filter(l => allLayers.includes(l)),
            ...allLayers.filter(l => !LAYER_ORDER.includes(l)).sort(),
        ];

        // Extract a short readable name from a full module ID.
        // Strips the common prefix up to and including the layer segment.
        // "pkg.dom.src.ctx.orchestration.slices.execute_workflow.Handler" → "execute_workflow.Handler"
        function shortName(m) {{
            const parts = m.id ? m.id.split('.') : m.name.split('.');
            const sliceIdx = parts.indexOf(m.slice);
            const layerIdx = parts.indexOf(m.layer);
            if (layerIdx >= 0 && layerIdx < parts.length - 1) {{
                return parts.slice(layerIdx + 1).join('.');
            }}
            if (sliceIdx >= 0 && sliceIdx < parts.length - 1) {{
                return parts.slice(sliceIdx + 1).join('.');
            }}
            // Fallback: last 2 segments
            return parts.slice(-2).join('.');
        }}

        // For slices layer, extract the feature name (first segment after "slices.")
        function featureName(m) {{
            const parts = m.id ? m.id.split('.') : m.name.split('.');
            const layerIdx = parts.indexOf(m.layer);
            if (layerIdx >= 0 && layerIdx + 1 < parts.length) {{
                return parts[layerIdx + 1];
            }}
            return shortName(m);
        }}

        // Build slice × layer matrix
        const matrix = {{}};
        const slices = new Set();

        MODULES.forEach(m => {{
            slices.add(m.slice);
            const key = `${{m.slice}}|${{m.layer}}`;
            if (!matrix[key]) {{
                matrix[key] = {{ modules: [], totalHealth: 0 }};
            }}
            matrix[key].modules.push(m);
            matrix[key].totalHealth += m.health;
        }});

        const sliceList = Array.from(slices).sort();

        function healthToColor(h) {{
            if (h >= 0.80) return '#00ff88';
            if (h >= 0.65) return '#44dd77';
            if (h >= 0.50) return '#88cc55';
            if (h >= 0.35) return '#ddaa33';
            if (h >= 0.20) return '#ff7744';
            return '#ff3333';
        }}

        // Group modules by feature within a cell
        function groupByFeature(modules) {{
            const groups = {{}};
            modules.forEach(m => {{
                const feat = featureName(m);
                if (!groups[feat]) groups[feat] = [];
                groups[feat].push(m);
            }});
            return groups;
        }}

        // Render table
        const table = document.getElementById('matrix');

        // Header row
        let headerRow = '<tr><th class="layer-header">Layer \\ Context</th>';
        sliceList.forEach(slice => {{
            const label = slice.split('.').pop() || slice;
            headerRow += `<th>${{label}}</th>`;
        }});
        headerRow += '</tr>';
        table.innerHTML = headerRow;

        // Data rows
        LAYERS.forEach(layer => {{
            let row = `<tr><td class="layer-label">${{layer}}</td>`;
            sliceList.forEach(slice => {{
                const key = `${{slice}}|${{layer}}`;
                const cell = matrix[key];

                if (cell && cell.modules.length > 0) {{
                    const avgHealth = cell.totalHealth / cell.modules.length;
                    const color = healthToColor(avgHealth);
                    const features = groupByFeature(cell.modules);
                    const featureNames = Object.keys(features).sort();
                    const showTags = featureNames.length <= 12;

                    let cellContent;
                    if (showTags && featureNames.length > 1) {{
                        // Show feature tags inline
                        const tags = featureNames.map(f => {{
                            const cnt = features[f].length;
                            const label = cnt > 1 ? `${{f}} (${{cnt}})` : f;
                            return `<span class="feature-tag">${{label}}</span>`;
                        }}).join('');
                        cellContent = `
                            <div class="cell-inner has-features" style="background:${{color}}20;border:1px solid ${{color}}">
                                <div class="cell-header">
                                    <span class="cell-count" style="color:${{color}}">${{cell.modules.length}}</span>
                                    <span class="cell-label">${{(avgHealth * 100).toFixed(0)}}%</span>
                                </div>
                                <div class="feature-tags">${{tags}}</div>
                            </div>
                        `;
                    }} else {{
                        cellContent = `
                            <div class="cell-inner" style="background:${{color}}20;border:1px solid ${{color}}">
                                <span class="cell-count" style="color:${{color}}">${{cell.modules.length}}</span>
                                <span class="cell-label">${{(avgHealth * 100).toFixed(0)}}%</span>
                            </div>
                        `;
                    }}

                    row += `<td class="cell" data-slice="${{slice}}" data-layer="${{layer}}">${{cellContent}}</td>`;
                }} else {{
                    row += '<td class="cell empty"><div class="cell-inner">-</div></td>';
                }}
            }});
            row += '</tr>';
            table.innerHTML += row;
        }});

        // Detail panel (click to inspect)
        const panel = document.getElementById('detail-panel');
        const panelContent = document.getElementById('detail-content');
        let selectedCell = null;

        function closePanel() {{
            panel.classList.remove('visible');
            document.body.classList.remove('has-panel');
            if (selectedCell) {{ selectedCell.classList.remove('selected'); selectedCell = null; }}
        }}

        panel.querySelector('.close-btn').addEventListener('click', closePanel);
        document.addEventListener('keydown', e => {{ if (e.key === 'Escape') closePanel(); }});

        document.querySelectorAll('.cell[data-slice]').forEach(cell => {{
            cell.addEventListener('click', () => {{
                const slice = cell.dataset.slice;
                const layer = cell.dataset.layer;
                const key = `${{slice}}|${{layer}}`;
                const data = matrix[key];
                if (!data) return;

                // Toggle selection
                if (selectedCell) selectedCell.classList.remove('selected');
                if (selectedCell === cell) {{ closePanel(); return; }}
                selectedCell = cell;
                cell.classList.add('selected');

                const features = groupByFeature(data.modules);
                const featureNames = Object.keys(features).sort();
                const avgHealth = data.totalHealth / data.modules.length;

                let body = `<div class="summary">${{data.modules.length}} modules · ${{featureNames.length}} features · ${{(avgHealth * 100).toFixed(0)}}% avg health</div>`;

                featureNames.forEach(feat => {{
                    const mods = features[feat];
                    const avgH = mods.reduce((s, m) => s + m.health, 0) / mods.length;
                    const hColor = healthToColor(avgH);
                    body += `<div class="feature-group">
                        <div class="feature-group-header"><span style="color:${{hColor}}">●</span> ${{feat}} <span style="color:#666;font-weight:400">(${{mods.length}})</span></div>`;
                    mods.forEach(m => {{
                        const sn = shortName(m);
                        if (sn !== feat) {{
                            body += `<div class="module-item">
                                <span style="color:${{m.color}}">●</span>
                                <span class="mod-name">${{sn}}</span>
                                <span style="color:#555;font-size:10px">${{(m.health * 100).toFixed(0)}}%</span>
                            </div>`;
                        }}
                    }});
                    body += '</div>';
                }});

                panelContent.innerHTML = `<h3>${{slice}} / ${{layer}}</h3>${{body}}`;
                panel.classList.add('visible');
                document.body.classList.add('has-panel');
            }});
        }});
    </script>
</body>
</html>"##,
        modules_json = modules_escaped
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_contains_doctype() {
        let html = generate("[]");
        assert!(html.starts_with("<!DOCTYPE html>"));
    }

    #[test]
    fn test_generate_contains_title() {
        let html = generate("[]");
        assert!(html.contains("<title>VSA Diagram"));
    }
}
