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
        th, td {{ padding: 12px 16px; text-align: center; border: 1px solid #222; min-width: 100px; }}
        th {{ background: #1a1a20; color: #888; font-weight: 500; position: sticky; top: 0; }}
        th.layer-header {{ writing-mode: horizontal-tb; background: #15151a; }}
        .layer-label {{ background: #15151a; font-weight: 500; text-align: left; color: #888; }}
        .cell {{ position: relative; cursor: pointer; transition: transform 0.2s; }}
        .cell:hover {{ transform: scale(1.05); z-index: 10; }}
        .cell-inner {{ border-radius: 6px; padding: 8px; min-height: 50px; display: flex; flex-direction: column; justify-content: center; align-items: center; }}
        .cell-count {{ font-size: 20px; font-weight: 600; }}
        .cell-label {{ font-size: 10px; color: rgba(255,255,255,0.6); margin-top: 4px; }}
        .empty {{ background: #0f0f12; color: #333; }}
        .legend {{ margin-top: 30px; display: flex; gap: 20px; flex-wrap: wrap; }}
        .legend-item {{ display: flex; align-items: center; gap: 8px; font-size: 12px; color: #888; }}
        .legend-color {{ width: 20px; height: 20px; border-radius: 4px; }}
        #tooltip {{ position: fixed; display: none; background: rgba(0,0,0,0.95); padding: 15px; border-radius: 8px; border: 1px solid #444; font-size: 12px; pointer-events: none; z-index: 200; max-width: 300px; }}
        #tooltip h3 {{ color: #00ff88; margin-bottom: 8px; }}
        #tooltip .module-list {{ max-height: 200px; overflow-y: auto; }}
        #tooltip .module-item {{ padding: 4px 0; border-bottom: 1px solid #222; }}
    </style>
</head>
<body>
    <h1>🍰 Vertical Slice Architecture</h1>
    <p class="subtitle">Columns = feature slices, Rows = architectural layers, Cells = module count</p>
    
    <div class="matrix-container">
        <table id="matrix"></table>
    </div>

    <div class="legend">
        <div class="legend-item"><div class="legend-color" style="background:#00ff88"></div>Excellent health</div>
        <div class="legend-item"><div class="legend-color" style="background:#88cc55"></div>OK health</div>
        <div class="legend-item"><div class="legend-color" style="background:#ff7744"></div>Poor health</div>
        <div class="legend-item"><div class="legend-color" style="background:#0f0f12;border:1px solid #333"></div>Empty (no modules)</div>
    </div>

    <div id="tooltip"></div>

    <script>
        const MODULES = {modules_json};
        const LAYERS = [...new Set(MODULES.map(m => m.layer))].sort();

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

        // Render table
        const table = document.getElementById('matrix');
        
        // Header row
        let headerRow = '<tr><th class="layer-header">Layer \\ Slice</th>';
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
                    row += `
                        <td class="cell" data-slice="${{slice}}" data-layer="${{layer}}">
                            <div class="cell-inner" style="background:${{color}}20;border:1px solid ${{color}}">
                                <span class="cell-count" style="color:${{color}}">${{cell.modules.length}}</span>
                                <span class="cell-label">${{(avgHealth * 100).toFixed(0)}}%</span>
                            </div>
                        </td>
                    `;
                }} else {{
                    row += '<td class="cell empty"><div class="cell-inner">-</div></td>';
                }}
            }});
            row += '</tr>';
            table.innerHTML += row;
        }});

        // Tooltips
        const tooltip = document.getElementById('tooltip');
        document.querySelectorAll('.cell[data-slice]').forEach(cell => {{
            cell.addEventListener('mouseenter', e => {{
                const slice = cell.dataset.slice;
                const layer = cell.dataset.layer;
                const key = `${{slice}}|${{layer}}`;
                const data = matrix[key];
                
                if (data) {{
                    tooltip.style.display = 'block';
                    tooltip.innerHTML = `
                        <h3>${{slice}} / ${{layer}}</h3>
                        <div class="module-list">
                            ${{data.modules.map(m => `
                                <div class="module-item">
                                    <span style="color:${{m.color}}">●</span> ${{m.name}}
                                    <span style="color:#666;font-size:10px">(${{(m.health * 100).toFixed(0)}}%)</span>
                                </div>
                            `).join('')}}
                        </div>
                    `;
                }}
            }});
            
            cell.addEventListener('mousemove', e => {{
                tooltip.style.left = (e.clientX + 15) + 'px';
                tooltip.style.top = (e.clientY + 15) + 'px';
            }});
            
            cell.addEventListener('mouseleave', () => {{
                tooltip.style.display = 'none';
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
