//! CodeCity Visualization
//!
//! 3D city metaphor where buildings represent modules.
//! - **Height** = cyclomatic complexity
//! - **Width** = function count
//! - **Color** = health score
//! - **Districts** = slices/packages

use crate::escape_json_for_html;

/// Generate CodeCity HTML visualization.
///
/// # Arguments
/// * `modules_json` - JSON array of module data with slice, layer, health, complexity
/// * `coupling_json` - JSON coupling matrix data
///
/// # Returns
/// Complete HTML document as a string
#[allow(clippy::uninlined_format_args)]
pub fn generate(modules_json: &str, coupling_json: &str) -> String {
    let modules_escaped = escape_json_for_html(modules_json);
    let coupling_escaped = escape_json_for_html(coupling_json);

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>CodeCity - Topology Visualization</title>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/three.js/r160/three.min.js"></script>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, sans-serif; background: #0a0a0f; color: #fff; overflow: hidden; }}
        #info {{ position: fixed; top: 20px; left: 20px; background: rgba(0,0,0,0.8); padding: 20px; border-radius: 12px; border: 1px solid #333; max-width: 300px; z-index: 100; }}
        #info h1 {{ font-size: 18px; margin-bottom: 10px; color: #00ff88; }}
        #info p {{ font-size: 12px; color: #888; margin-bottom: 8px; }}
        #legend {{ margin-top: 15px; }}
        .legend-item {{ display: flex; align-items: center; gap: 8px; margin: 4px 0; font-size: 11px; }}
        .legend-color {{ width: 16px; height: 16px; border-radius: 3px; }}
        #tooltip {{ position: fixed; display: none; background: rgba(0,0,0,0.95); padding: 15px; border-radius: 8px; border: 1px solid #444; font-size: 12px; pointer-events: none; z-index: 200; max-width: 320px; }}
        #tooltip h3 {{ color: #00ff88; margin-bottom: 8px; font-size: 14px; }}
        #tooltip .metric {{ display: flex; justify-content: space-between; padding: 3px 0; border-bottom: 1px solid #222; }}
        #tooltip .metric:last-child {{ border-bottom: none; }}
        #tooltip .label {{ color: #888; }}
        #tooltip .value {{ color: #fff; font-weight: 500; }}
        #controls {{ position: fixed; bottom: 20px; left: 20px; background: rgba(0,0,0,0.8); padding: 12px 16px; border-radius: 8px; font-size: 11px; color: #666; }}
    </style>
</head>
<body>
    <div id="info">
        <h1>🏙️ CodeCity</h1>
        <p>Buildings represent modules. Height = complexity, color = health.</p>
        <div id="legend">
            <div class="legend-item"><div class="legend-color" style="background:#00ff88"></div>Excellent (≥80%)</div>
            <div class="legend-item"><div class="legend-color" style="background:#44dd77"></div>Good (≥65%)</div>
            <div class="legend-item"><div class="legend-color" style="background:#88cc55"></div>OK (≥50%)</div>
            <div class="legend-item"><div class="legend-color" style="background:#ddaa33"></div>Warning (≥35%)</div>
            <div class="legend-item"><div class="legend-color" style="background:#ff7744"></div>Poor (≥20%)</div>
            <div class="legend-item"><div class="legend-color" style="background:#ff3333"></div>Critical (&lt;20%)</div>
        </div>
    </div>
    <div id="tooltip"></div>
    <div id="controls">🖱️ Drag to rotate • Scroll to zoom • Right-click to pan</div>

    <script>
        const MODULES = {modules_json};
        const COUPLING = {coupling_json};

        // Scene setup
        const scene = new THREE.Scene();
        scene.background = new THREE.Color(0x0a0a0f);
        
        const camera = new THREE.PerspectiveCamera(60, window.innerWidth / window.innerHeight, 0.1, 1000);
        camera.position.set(30, 40, 50);
        camera.lookAt(0, 0, 0);

        const renderer = new THREE.WebGLRenderer({{ antialias: true }});
        renderer.setSize(window.innerWidth, window.innerHeight);
        renderer.setPixelRatio(window.devicePixelRatio);
        document.body.appendChild(renderer.domElement);

        // Lighting
        const ambient = new THREE.AmbientLight(0x404040, 0.5);
        scene.add(ambient);
        const directional = new THREE.DirectionalLight(0xffffff, 0.8);
        directional.position.set(50, 100, 50);
        scene.add(directional);

        // Ground plane
        const groundGeo = new THREE.PlaneGeometry(200, 200);
        const groundMat = new THREE.MeshStandardMaterial({{ color: 0x111115, roughness: 1 }});
        const ground = new THREE.Mesh(groundGeo, groundMat);
        ground.rotation.x = -Math.PI / 2;
        ground.position.y = -0.1;
        scene.add(ground);

        // Grid
        const grid = new THREE.GridHelper(100, 50, 0x222222, 0x181818);
        scene.add(grid);

        // Group modules by slice (district)
        const slices = {{}};
        MODULES.forEach(m => {{
            if (!slices[m.slice]) slices[m.slice] = [];
            slices[m.slice].push(m);
        }});

        // Layout districts
        const districtNames = Object.keys(slices);
        const gridSize = Math.ceil(Math.sqrt(districtNames.length));
        const districtSpacing = 25;
        const buildingSpacing = 3;

        const buildings = [];
        let districtIndex = 0;

        districtNames.forEach(sliceName => {{
            const modules = slices[sliceName];
            const districtX = (districtIndex % gridSize) * districtSpacing - (gridSize * districtSpacing) / 2;
            const districtZ = Math.floor(districtIndex / gridSize) * districtSpacing - (gridSize * districtSpacing) / 2;

            // Layout buildings in district
            const buildingGrid = Math.ceil(Math.sqrt(modules.length));
            modules.forEach((m, i) => {{
                const localX = (i % buildingGrid) * buildingSpacing - (buildingGrid * buildingSpacing) / 2;
                const localZ = Math.floor(i / buildingGrid) * buildingSpacing - (buildingGrid * buildingSpacing) / 2;

                // Building dimensions based on metrics (log scale for height to prevent extreme outliers)
                const rawHeight = m.total_cyclomatic || 1;
                const height = Math.max(1, Math.log10(rawHeight + 1) * 3);
                const width = Math.max(0.8, Math.sqrt(m.function_count) * 0.5);
                const depth = width;

                const geometry = new THREE.BoxGeometry(width, height, depth);
                const material = new THREE.MeshStandardMaterial({{
                    color: new THREE.Color(m.color),
                    roughness: 0.6,
                    metalness: 0.2
                }});
                const building = new THREE.Mesh(geometry, material);
                building.position.set(districtX + localX, height / 2, districtZ + localZ);
                building.userData = m;
                scene.add(building);
                buildings.push(building);
            }});

            districtIndex++;
        }});

        // Simple orbit controls
        let isDragging = false;
        let isPanning = false;
        let previousMouse = {{ x: 0, y: 0 }};
        let spherical = {{ radius: 80, theta: Math.PI / 4, phi: Math.PI / 3 }};
        let target = new THREE.Vector3(0, 0, 0);

        function updateCamera() {{
            camera.position.x = target.x + spherical.radius * Math.sin(spherical.phi) * Math.cos(spherical.theta);
            camera.position.y = target.y + spherical.radius * Math.cos(spherical.phi);
            camera.position.z = target.z + spherical.radius * Math.sin(spherical.phi) * Math.sin(spherical.theta);
            camera.lookAt(target);
        }}
        updateCamera();

        renderer.domElement.addEventListener('mousedown', e => {{
            if (e.button === 0) isDragging = true;
            if (e.button === 2) isPanning = true;
            previousMouse = {{ x: e.clientX, y: e.clientY }};
        }});

        renderer.domElement.addEventListener('mousemove', e => {{
            const deltaX = e.clientX - previousMouse.x;
            const deltaY = e.clientY - previousMouse.y;

            if (isDragging) {{
                spherical.theta -= deltaX * 0.01;
                spherical.phi -= deltaY * 0.01;
                spherical.phi = Math.max(0.1, Math.min(Math.PI - 0.1, spherical.phi));
                updateCamera();
            }}
            if (isPanning) {{
                const right = new THREE.Vector3();
                const up = new THREE.Vector3(0, 1, 0);
                camera.getWorldDirection(right);
                right.cross(up).normalize();
                target.add(right.multiplyScalar(-deltaX * 0.1));
                target.y += deltaY * 0.1;
                updateCamera();
            }}
            previousMouse = {{ x: e.clientX, y: e.clientY }};
        }});

        window.addEventListener('mouseup', () => {{ isDragging = false; isPanning = false; }});
        renderer.domElement.addEventListener('contextmenu', e => e.preventDefault());

        renderer.domElement.addEventListener('wheel', e => {{
            spherical.radius *= 1 + e.deltaY * 0.001;
            spherical.radius = Math.max(10, Math.min(200, spherical.radius));
            updateCamera();
        }});

        // Raycasting for tooltips
        const raycaster = new THREE.Raycaster();
        const mouse = new THREE.Vector2();
        const tooltip = document.getElementById('tooltip');

        renderer.domElement.addEventListener('mousemove', e => {{
            mouse.x = (e.clientX / window.innerWidth) * 2 - 1;
            mouse.y = -(e.clientY / window.innerHeight) * 2 + 1;

            raycaster.setFromCamera(mouse, camera);
            const intersects = raycaster.intersectObjects(buildings);

            if (intersects.length > 0) {{
                const m = intersects[0].object.userData;
                tooltip.style.display = 'block';
                tooltip.style.left = (e.clientX + 15) + 'px';
                tooltip.style.top = (e.clientY + 15) + 'px';
                tooltip.innerHTML = `
                    <h3>${{m.name}}</h3>
                    <div class="metric"><span class="label">Slice</span><span class="value">${{m.slice}}</span></div>
                    <div class="metric"><span class="label">Layer</span><span class="value">${{m.layer}}</span></div>
                    <div class="metric"><span class="label">Functions</span><span class="value">${{m.function_count}}</span></div>
                    <div class="metric"><span class="label">Complexity</span><span class="value">${{m.total_cyclomatic}}</span></div>
                    <div class="metric"><span class="label">Cognitive</span><span class="value">${{m.total_cognitive}}</span></div>
                    <div class="metric"><span class="label">LOC</span><span class="value">${{m.lines_of_code}}</span></div>
                    <div class="metric"><span class="label">Coupling (Ca/Ce)</span><span class="value">${{m.ca}} / ${{m.ce}}</span></div>
                    <div class="metric"><span class="label">Health</span><span class="value" style="color:${{m.color}}">${{(m.health * 100).toFixed(0)}}% (${{m.health_label}})</span></div>
                `;
            }} else {{
                tooltip.style.display = 'none';
            }}
        }});

        // Resize handler
        window.addEventListener('resize', () => {{
            camera.aspect = window.innerWidth / window.innerHeight;
            camera.updateProjectionMatrix();
            renderer.setSize(window.innerWidth, window.innerHeight);
        }});

        // Animation loop
        function animate() {{
            requestAnimationFrame(animate);
            renderer.render(scene, camera);
        }}
        animate();
    </script>
</body>
</html>"##,
        modules_json = modules_escaped,
        coupling_json = coupling_escaped
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
        assert!(html.contains("<title>CodeCity"));
    }
}
