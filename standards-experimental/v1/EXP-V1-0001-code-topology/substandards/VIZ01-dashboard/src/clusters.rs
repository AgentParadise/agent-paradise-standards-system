//! Package Clusters Visualization
//!
//! 2D force-directed graph showing package/slice relationships.
//! - **Nodes** = packages/slices
//! - **Edges** = coupling between packages
//! - **Size** = module count
//! - **Color** = average health

/// Generate Package Clusters HTML visualization.
///
/// # Arguments
/// * `modules_json` - JSON array of module data
/// * `coupling_json` - JSON coupling matrix data
///
/// # Returns
/// Complete HTML document as a string
#[allow(clippy::uninlined_format_args)]
pub fn generate(modules_json: &str, coupling_json: &str) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Package Clusters - Topology Visualization</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, sans-serif; background: #0a0a0f; color: #fff; overflow: hidden; display: flex; }}
        #main {{ flex: 1; position: relative; }}
        canvas {{ display: block; }}
        #info {{ position: fixed; top: 20px; left: 20px; background: rgba(0,0,0,0.8); padding: 20px; border-radius: 12px; border: 1px solid #333; max-width: 280px; z-index: 100; }}
        #info h1 {{ font-size: 18px; margin-bottom: 10px; color: #00ff88; }}
        #info p {{ font-size: 12px; color: #888; margin-bottom: 8px; }}
        #filter {{ margin-top: 15px; padding-top: 15px; border-top: 1px solid #333; }}
        #filter label {{ display: block; font-size: 11px; color: #888; margin-bottom: 6px; }}
        #filter input[type="range"] {{ width: 100%; cursor: pointer; }}
        #filter-value {{ float: right; color: #00ff88; }}
        #sidebar {{ width: 280px; background: #111116; border-left: 1px solid #222; overflow-y: auto; padding: 15px; }}
        #sidebar h2 {{ font-size: 14px; color: #888; margin-bottom: 15px; display: flex; justify-content: space-between; }}
        .package-item {{ padding: 10px; margin-bottom: 8px; background: #1a1a20; border-radius: 8px; cursor: pointer; transition: all 0.2s; border: 1px solid transparent; }}
        .package-item:hover {{ border-color: #444; }}
        .package-item.selected {{ border-color: #00ff88; background: #1a2a20; }}
        .package-name {{ font-weight: 500; margin-bottom: 4px; font-size: 13px; }}
        .package-stats {{ font-size: 11px; color: #666; display: flex; gap: 12px; }}
        .package-coupling {{ color: #88aaff; }}
        .package-health {{ }}
        #tooltip {{ position: fixed; display: none; background: rgba(0,0,0,0.95); padding: 15px; border-radius: 8px; border: 1px solid #444; font-size: 12px; pointer-events: none; z-index: 200; }}
        #tooltip h3 {{ color: #00ff88; margin-bottom: 8px; }}
        #controls {{ position: fixed; bottom: 20px; left: 20px; background: rgba(0,0,0,0.8); padding: 12px 16px; border-radius: 8px; font-size: 11px; color: #666; }}
    </style>
</head>
<body>
    <div id="main">
        <div id="info">
            <h1>🔧 Package Clusters</h1>
            <p>Circles = packages. Lines = coupling strength.</p>
            <div id="filter">
                <label>Coupling Threshold <span id="filter-value">0%</span></label>
                <input type="range" id="coupling-slider" min="0" max="100" value="0">
            </div>
        </div>
        <div id="tooltip"></div>
        <div id="controls">🖱️ Drag to pan • Scroll to zoom • Click sidebar to focus</div>
        <canvas id="canvas"></canvas>
    </div>
    <div id="sidebar">
        <h2>Packages <span id="package-count"></span></h2>
        <div id="package-list"></div>
    </div>

    <script>
        const MODULES = {modules_json};
        const COUPLING = {coupling_json};

        const canvas = document.getElementById('canvas');
        const ctx = canvas.getContext('2d');
        let width, height;
        let couplingThreshold = 0;
        let selectedSlice = null;

        function resize() {{
            width = canvas.parentElement.clientWidth;
            height = window.innerHeight;
            canvas.width = width * devicePixelRatio;
            canvas.height = height * devicePixelRatio;
            canvas.style.width = width + 'px';
            canvas.style.height = height + 'px';
            ctx.scale(devicePixelRatio, devicePixelRatio);
        }}
        resize();
        window.addEventListener('resize', () => {{ resize(); draw(); }});

        // Group modules by slice
        const sliceMap = {{}};
        MODULES.forEach(m => {{
            if (!sliceMap[m.slice]) {{
                sliceMap[m.slice] = {{ modules: [], totalHealth: 0 }};
            }}
            sliceMap[m.slice].modules.push(m);
            sliceMap[m.slice].totalHealth += m.health;
        }});

        // Build slice nodes
        const slices = Object.entries(sliceMap).map(([name, data], i) => {{
            const avgHealth = data.totalHealth / data.modules.length;
            return {{
                name,
                modules: data.modules,
                count: data.modules.length,
                health: avgHealth,
                color: healthToColor(avgHealth),
                x: width / 2 + (Math.random() - 0.5) * 200,
                y: height / 2 + (Math.random() - 0.5) * 200,
                vx: 0,
                vy: 0,
                radius: Math.max(25, Math.sqrt(data.modules.length) * 15),
                totalCoupling: 0
            }};
        }});

        // Build edges from coupling matrix
        const edges = [];
        const moduleToSlice = {{}};
        MODULES.forEach(m => moduleToSlice[m.id] = m.slice);

        for (let i = 0; i < COUPLING.modules.length; i++) {{
            for (let j = i + 1; j < COUPLING.modules.length; j++) {{
                const strength = COUPLING.matrix[i][j] + COUPLING.matrix[j][i];
                if (strength > 0) {{
                    const sliceA = moduleToSlice[COUPLING.modules[i]];
                    const sliceB = moduleToSlice[COUPLING.modules[j]];
                    if (sliceA && sliceB && sliceA !== sliceB) {{
                        let edge = edges.find(e => 
                            (e.source === sliceA && e.target === sliceB) ||
                            (e.source === sliceB && e.target === sliceA)
                        );
                        if (!edge) {{
                            edges.push({{ source: sliceA, target: sliceB, strength: strength }});
                        }} else {{
                            edge.strength += strength;
                        }}
                    }}
                }}
            }}
        }}

        // Calculate total coupling per slice
        const maxStrength = Math.max(...edges.map(e => e.strength), 1);
        edges.forEach(edge => {{
            const sourceSlice = slices.find(s => s.name === edge.source);
            const targetSlice = slices.find(s => s.name === edge.target);
            if (sourceSlice) sourceSlice.totalCoupling += edge.strength;
            if (targetSlice) targetSlice.totalCoupling += edge.strength;
        }});

        // Sort slices by coupling (descending)
        const sortedSlices = [...slices].sort((a, b) => b.totalCoupling - a.totalCoupling);

        // Populate sidebar
        function renderSidebar() {{
            const list = document.getElementById('package-list');
            document.getElementById('package-count').textContent = `(${{slices.length}})`;
            
            list.innerHTML = sortedSlices.map(s => `
                <div class="package-item ${{selectedSlice === s.name ? 'selected' : ''}}" data-slice="${{s.name}}">
                    <div class="package-name" style="color:${{s.color}}">${{s.name}}</div>
                    <div class="package-stats">
                        <span class="package-coupling">⚡ ${{s.totalCoupling.toFixed(1)}}</span>
                        <span class="package-health" style="color:${{s.color}}">❤ ${{(s.health * 100).toFixed(0)}}%</span>
                        <span>📦 ${{s.count}}</span>
                    </div>
                </div>
            `).join('');

            // Add click handlers
            list.querySelectorAll('.package-item').forEach(item => {{
                item.addEventListener('click', () => {{
                    const name = item.dataset.slice;
                    selectedSlice = selectedSlice === name ? null : name;
                    renderSidebar();
                    
                    // Center on selected slice
                    if (selectedSlice) {{
                        const slice = slices.find(s => s.name === selectedSlice);
                        if (slice) {{
                            transform.x = width / 2 - slice.x * transform.scale;
                            transform.y = height / 2 - slice.y * transform.scale;
                        }}
                    }}
                }});
            }});
        }}
        renderSidebar();

        // Coupling slider
        document.getElementById('coupling-slider').addEventListener('input', e => {{
            couplingThreshold = parseInt(e.target.value) / 100;
            document.getElementById('filter-value').textContent = `${{e.target.value}}%`;
        }});

        function healthToColor(h) {{
            if (h >= 0.80) return '#00ff88';
            if (h >= 0.65) return '#44dd77';
            if (h >= 0.50) return '#88cc55';
            if (h >= 0.35) return '#ddaa33';
            if (h >= 0.20) return '#ff7744';
            return '#ff3333';
        }}

        let transform = {{ x: 0, y: 0, scale: 1 }};
        
        function simulate() {{
            const centerX = width / 2;
            const centerY = height / 2;

            slices.forEach(node => {{
                node.vx += (centerX - node.x) * 0.0005;
                node.vy += (centerY - node.y) * 0.0005;

                slices.forEach(other => {{
                    if (node === other) return;
                    const dx = node.x - other.x;
                    const dy = node.y - other.y;
                    const dist = Math.sqrt(dx * dx + dy * dy) || 1;
                    const minDist = node.radius + other.radius + 30;
                    if (dist < minDist) {{
                        const force = (minDist - dist) / dist * 0.05;
                        node.vx += dx * force;
                        node.vy += dy * force;
                    }}
                }});
            }});

            edges.forEach(edge => {{
                const source = slices.find(s => s.name === edge.source);
                const target = slices.find(s => s.name === edge.target);
                if (!source || !target) return;

                const dx = target.x - source.x;
                const dy = target.y - source.y;
                const dist = Math.sqrt(dx * dx + dy * dy) || 1;
                const idealDist = 150;
                const force = (dist - idealDist) * 0.0001 * edge.strength;

                source.vx += dx / dist * force;
                source.vy += dy / dist * force;
                target.vx -= dx / dist * force;
                target.vy -= dy / dist * force;
            }});

            slices.forEach(node => {{
                node.vx *= 0.9;
                node.vy *= 0.9;
                node.x += node.vx;
                node.y += node.vy;
            }});
        }}

        function draw() {{
            ctx.clearRect(0, 0, width, height);
            ctx.save();
            ctx.translate(transform.x, transform.y);
            ctx.scale(transform.scale, transform.scale);

            // Filter edges by threshold
            const thresholdValue = couplingThreshold * maxStrength;
            const visibleEdges = edges.filter(e => e.strength >= thresholdValue);

            // Draw edges
            visibleEdges.forEach(edge => {{
                const source = slices.find(s => s.name === edge.source);
                const target = slices.find(s => s.name === edge.target);
                if (!source || !target) return;

                const isHighlighted = selectedSlice && (edge.source === selectedSlice || edge.target === selectedSlice);
                const opacity = isHighlighted ? 0.8 : (0.1 + (edge.strength / maxStrength) * 0.5);
                const lineWidth = isHighlighted ? 3 : (1 + (edge.strength / maxStrength) * 3);

                ctx.beginPath();
                ctx.moveTo(source.x, source.y);
                ctx.lineTo(target.x, target.y);
                ctx.strokeStyle = isHighlighted ? '#00ff88' : `rgba(100, 100, 150, ${{opacity}})`;
                ctx.lineWidth = lineWidth;
                ctx.stroke();
            }});

            // Draw nodes
            slices.forEach(node => {{
                const isSelected = node.name === selectedSlice;
                const hasVisibleEdge = visibleEdges.some(e => e.source === node.name || e.target === node.name);
                const alpha = (couplingThreshold > 0 && !hasVisibleEdge && !isSelected) ? 0.3 : 1;

                // Glow
                const gradient = ctx.createRadialGradient(node.x, node.y, 0, node.x, node.y, node.radius * 1.5);
                gradient.addColorStop(0, node.color + (isSelected ? '80' : '40'));
                gradient.addColorStop(1, 'transparent');
                ctx.beginPath();
                ctx.arc(node.x, node.y, node.radius * (isSelected ? 2 : 1.5), 0, Math.PI * 2);
                ctx.fillStyle = gradient;
                ctx.globalAlpha = alpha;
                ctx.fill();

                // Circle
                ctx.beginPath();
                ctx.arc(node.x, node.y, node.radius, 0, Math.PI * 2);
                ctx.fillStyle = node.color + '30';
                ctx.fill();
                ctx.strokeStyle = isSelected ? '#fff' : node.color;
                ctx.lineWidth = isSelected ? 3 : 2;
                ctx.stroke();

                // Label
                ctx.fillStyle = '#fff';
                ctx.font = (isSelected ? 'bold ' : '') + '12px -apple-system, sans-serif';
                ctx.textAlign = 'center';
                ctx.textBaseline = 'middle';
                const label = node.name.split('::').pop() || node.name.split('.').pop() || node.name;
                ctx.fillText(label, node.x, node.y);
                ctx.globalAlpha = 1;
            }});

            ctx.restore();
        }}

        function animate() {{
            simulate();
            draw();
            requestAnimationFrame(animate);
        }}
        animate();

        let isDragging = false;
        let lastMouse = {{ x: 0, y: 0 }};

        canvas.addEventListener('mousedown', e => {{
            isDragging = true;
            lastMouse = {{ x: e.clientX, y: e.clientY }};
        }});

        canvas.addEventListener('mousemove', e => {{
            if (isDragging) {{
                transform.x += e.clientX - lastMouse.x;
                transform.y += e.clientY - lastMouse.y;
                lastMouse = {{ x: e.clientX, y: e.clientY }};
            }}

            const tooltip = document.getElementById('tooltip');
            const mx = (e.clientX - transform.x) / transform.scale;
            const my = (e.clientY - transform.y) / transform.scale;

            const hovered = slices.find(s => {{
                const dx = s.x - mx;
                const dy = s.y - my;
                return Math.sqrt(dx*dx + dy*dy) < s.radius;
            }});

            if (hovered) {{
                tooltip.style.display = 'block';
                tooltip.style.left = (e.clientX + 15) + 'px';
                tooltip.style.top = (e.clientY + 15) + 'px';
                tooltip.innerHTML = `
                    <h3>${{hovered.name}}</h3>
                    <div>Modules: ${{hovered.count}}</div>
                    <div>Coupling: ${{hovered.totalCoupling.toFixed(1)}}</div>
                    <div style="color:${{hovered.color}}">Health: ${{(hovered.health * 100).toFixed(0)}}%</div>
                    <div style="margin-top:8px;color:#666;font-size:11px">
                        ${{hovered.modules.slice(0, 5).map(m => m.name).join(', ')}}${{hovered.modules.length > 5 ? '...' : ''}}
                    </div>
                `;
            }} else {{
                tooltip.style.display = 'none';
            }}
        }});

        canvas.addEventListener('mouseup', () => isDragging = false);
        canvas.addEventListener('mouseleave', () => isDragging = false);

        canvas.addEventListener('wheel', e => {{
            e.preventDefault();
            const scale = transform.scale * (1 - e.deltaY * 0.001);
            transform.scale = Math.max(0.2, Math.min(3, scale));
        }});
    </script>
</body>
</html>"##,
        modules_json = modules_json,
        coupling_json = coupling_json
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_contains_doctype() {
        let html = generate("[]", "{}");
        assert!(html.starts_with("<!DOCTYPE html>"));
    }

    #[test]
    fn test_generate_contains_title() {
        let html = generate("[]", "{}");
        assert!(html.contains("<title>Package Clusters"));
    }
}
