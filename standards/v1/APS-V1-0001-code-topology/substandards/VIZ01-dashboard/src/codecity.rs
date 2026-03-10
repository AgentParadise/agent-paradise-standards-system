//! CodeCity Visualization
//!
//! 3D city metaphor where buildings represent modules.
//! - **Height** = cyclomatic complexity (log-scaled)
//! - **Footprint** = lines of code (sqrt-scaled)
//! - **Color** = health score (green → red gradient)
//! - **Districts** = slices/packages (treemap layout with labeled ground planes)

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
    let _coupling_escaped = escape_json_for_html(coupling_json);

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>CodeCity - Topology Visualization</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, sans-serif; background: #0a0a0f; color: #fff; overflow: hidden; }}
        #info {{ position: fixed; top: 20px; left: 20px; background: rgba(0,0,0,0.85); padding: 20px; border-radius: 12px; border: 1px solid #333; max-width: 320px; z-index: 100; backdrop-filter: blur(10px); }}
        #info h1 {{ font-size: 18px; margin-bottom: 6px; color: #00ff88; }}
        #info .subtitle {{ font-size: 11px; color: #666; margin-bottom: 12px; }}
        #info p {{ font-size: 12px; color: #888; margin-bottom: 6px; }}
        #legend {{ margin-top: 12px; }}
        .legend-section {{ margin-bottom: 10px; }}
        .legend-title {{ font-size: 11px; color: #666; margin-bottom: 4px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; }}
        .legend-item {{ display: flex; align-items: center; gap: 8px; margin: 3px 0; font-size: 11px; color: #aaa; }}
        .legend-color {{ width: 14px; height: 14px; border-radius: 3px; flex-shrink: 0; }}
        #tooltip {{ position: fixed; display: none; background: rgba(10,10,20,0.95); padding: 16px; border-radius: 10px; border: 1px solid #444; font-size: 12px; pointer-events: none; z-index: 200; max-width: 360px; min-width: 260px; backdrop-filter: blur(10px); box-shadow: 0 8px 32px rgba(0,0,0,0.5); }}
        #tooltip h3 {{ color: #6bf; margin-bottom: 10px; font-size: 14px; word-break: break-word; }}
        #tooltip .section-label {{ color: #555; font-size: 10px; text-transform: uppercase; letter-spacing: 0.5px; margin-top: 8px; margin-bottom: 4px; }}
        #tooltip .metric {{ display: flex; justify-content: space-between; padding: 3px 0; border-bottom: 1px solid #1a1a2a; }}
        #tooltip .metric:last-child {{ border-bottom: none; }}
        #tooltip .label {{ color: #888; }}
        #tooltip .value {{ color: #fff; font-weight: 500; }}
        #tooltip .health-bar {{ height: 6px; border-radius: 3px; background: #222; margin-top: 8px; overflow: hidden; }}
        #tooltip .health-fill {{ height: 100%; border-radius: 3px; transition: width 0.3s; }}
        #tooltip .health-breakdown {{ margin-top: 6px; }}
        #tooltip .breakdown-item {{ display: flex; justify-content: space-between; font-size: 10px; padding: 2px 0; color: #777; }}
        #tooltip .breakdown-item .score {{ font-weight: 500; }}
        #controls {{ position: fixed; bottom: 20px; left: 20px; background: rgba(0,0,0,0.8); padding: 12px 16px; border-radius: 8px; font-size: 11px; color: #666; }}
        #minimap {{ position: fixed; bottom: 20px; right: 20px; width: 180px; height: 180px; background: rgba(0,0,0,0.85); border-radius: 10px; border: 1px solid #333; overflow: hidden; z-index: 100; }}
        #minimap canvas {{ width: 100%; height: 100%; }}
    </style>
</head>
<body>
    <div id="info">
        <h1>🏙️ CodeCity</h1>
        <div class="subtitle" id="stats"></div>
        <p>Buildings = modules. Districts = packages.</p>
        <div id="legend">
            <div class="legend-section">
                <div class="legend-title">Height → Complexity</div>
                <div class="legend-item"><div class="legend-color" style="background:linear-gradient(to top, #333, #888)"></div>Low → High cyclomatic complexity</div>
            </div>
            <div class="legend-section">
                <div class="legend-title">Footprint → Lines of Code</div>
                <div class="legend-item"><div class="legend-color" style="background:#555; width:8px; height:8px;"></div>Small module</div>
                <div class="legend-item"><div class="legend-color" style="background:#555; width:14px; height:14px;"></div>Large module</div>
            </div>
            <div class="legend-section">
                <div class="legend-title">Color → Health Score</div>
                <div class="legend-item"><div class="legend-color" style="background:#00ff88"></div>Excellent (≥80%)</div>
                <div class="legend-item"><div class="legend-color" style="background:#88cc55"></div>OK (≥50%)</div>
                <div class="legend-item"><div class="legend-color" style="background:#ddaa33"></div>Warning (≥35%)</div>
                <div class="legend-item"><div class="legend-color" style="background:#ff3333"></div>Critical (&lt;20%)</div>
            </div>
        </div>
    </div>
    <div id="tooltip"></div>
    <div id="controls">🖱️ Left-drag: rotate • Right-drag: pan • Scroll: zoom • Click: select</div>
    <div id="minimap"><canvas id="minimap-canvas"></canvas></div>

    <script type="importmap">
    {{
        "imports": {{
            "three": "https://cdn.jsdelivr.net/npm/three@0.160.0/build/three.module.js",
            "three/addons/": "https://cdn.jsdelivr.net/npm/three@0.160.0/examples/jsm/"
        }}
    }}
    </script>
    <script type="module">
        import * as THREE from 'three';
        import {{ CSS2DRenderer, CSS2DObject }} from 'three/addons/renderers/CSS2DRenderer.js';

        const MODULES = {modules_json};

        // ====================================================================
        // Squarified Treemap Layout
        // ====================================================================
        function treemapLayout(items, x, y, w, h) {{
            if (items.length === 0) return [];
            if (items.length === 1) {{
                return [{{ item: items[0], x, y, w, h }}];
            }}

            // Sort descending by value
            const sorted = [...items].sort((a, b) => b.value - a.value);
            const total = sorted.reduce((s, i) => s + i.value, 0);
            if (total === 0) return sorted.map((item, i) => ({{ item, x: x + i * 0.1, y, w: 0.1, h: 0.1 }}));

            // Find best split
            let sum = 0;
            let splitIdx = 0;
            const half = total / 2;
            for (let i = 0; i < sorted.length; i++) {{
                sum += sorted[i].value;
                if (sum >= half) {{ splitIdx = i + 1; break; }}
            }}
            splitIdx = Math.max(1, Math.min(splitIdx, sorted.length - 1));

            const left = sorted.slice(0, splitIdx);
            const right = sorted.slice(splitIdx);
            const leftVal = left.reduce((s, i) => s + i.value, 0);
            const ratio = leftVal / total;

            let results = [];
            if (w >= h) {{
                // Split horizontally
                const splitW = w * ratio;
                results = results.concat(treemapLayout(left, x, y, splitW, h));
                results = results.concat(treemapLayout(right, x + splitW, y, w - splitW, h));
            }} else {{
                // Split vertically
                const splitH = h * ratio;
                results = results.concat(treemapLayout(left, x, y, w, splitH));
                results = results.concat(treemapLayout(right, x, y + splitH, w, h - splitH));
            }}
            return results;
        }}

        // ====================================================================
        // Scene Setup
        // ====================================================================
        const scene = new THREE.Scene();
        scene.background = new THREE.Color(0x0a0a0f);
        scene.fog = new THREE.FogExp2(0x0a0a0f, 0.003);

        const camera = new THREE.PerspectiveCamera(55, window.innerWidth / window.innerHeight, 0.1, 2000);

        const renderer = new THREE.WebGLRenderer({{ antialias: true }});
        renderer.setSize(window.innerWidth, window.innerHeight);
        renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
        renderer.shadowMap.enabled = true;
        renderer.shadowMap.type = THREE.PCFSoftShadowMap;
        document.body.appendChild(renderer.domElement);

        // CSS2D for district labels
        const labelRenderer = new CSS2DRenderer();
        labelRenderer.setSize(window.innerWidth, window.innerHeight);
        labelRenderer.domElement.style.position = 'absolute';
        labelRenderer.domElement.style.top = '0px';
        labelRenderer.domElement.style.pointerEvents = 'none';
        document.body.appendChild(labelRenderer.domElement);

        // Lighting
        const ambient = new THREE.AmbientLight(0xffffff, 0.35);
        scene.add(ambient);
        const sun = new THREE.DirectionalLight(0xffffff, 1.0);
        sun.position.set(80, 120, 60);
        sun.castShadow = true;
        sun.shadow.mapSize.width = 2048;
        sun.shadow.mapSize.height = 2048;
        sun.shadow.camera.near = 1;
        sun.shadow.camera.far = 400;
        sun.shadow.camera.left = -200;
        sun.shadow.camera.right = 200;
        sun.shadow.camera.top = 200;
        sun.shadow.camera.bottom = -200;
        scene.add(sun);

        const fill = new THREE.DirectionalLight(0x4466aa, 0.3);
        fill.position.set(-40, 30, -60);
        scene.add(fill);

        // ====================================================================
        // Process Data: Group by top-level package (fewer, larger districts)
        // ====================================================================
        function getTopPackage(slice) {{
            // Group to top-level: packages.syn-domain, lib::agentic-primitives, etc.
            const parts = slice.replace(/::/g, '.').split('.');
            return parts.slice(0, 2).join('.');
        }}

        const packageGroups = {{}};
        MODULES.forEach(m => {{
            const pkg = getTopPackage(m.slice);
            if (!packageGroups[pkg]) packageGroups[pkg] = [];
            packageGroups[pkg].push(m);
        }});

        const packageNames = Object.keys(packageGroups).sort((a, b) =>
            packageGroups[b].length - packageGroups[a].length
        );

        // Stats
        document.getElementById('stats').textContent =
            `${{MODULES.length}} modules • ${{packageNames.length}} districts • ${{MODULES.reduce((s,m) => s + m.lines_of_code, 0).toLocaleString()}} LOC`;

        // ====================================================================
        // Treemap Layout for Districts
        // ====================================================================
        const totalLOC = MODULES.reduce((s, m) => s + (m.lines_of_code || 1), 0);
        const citySize = Math.sqrt(totalLOC) * 0.15;  // Scale city to LOC

        const districtItems = packageNames.map(name => ({{
            name,
            value: packageGroups[name].reduce((s, m) => s + (m.lines_of_code || 1), 0)
        }}));

        const districtRects = treemapLayout(districtItems, -citySize/2, -citySize/2, citySize, citySize);

        // ====================================================================
        // District ground colors (by top-level package)
        // ====================================================================
        const districtColors = [
            0x1a1a2e, 0x16213e, 0x1a2332, 0x1e1e30, 0x1c2833,
            0x212130, 0x1a2a1a, 0x2a1a2a, 0x2a2a1a, 0x1a2a2a,
            0x261a2e, 0x1e2e1a, 0x2e1e1a, 0x1a1e2e, 0x2e2e1a,
        ];

        // ====================================================================
        // Build the City
        // ====================================================================
        const buildings = [];
        const districtGroups = [];
        const PADDING = 0.8; // District padding ratio
        const GAP = 1.5; // Gap between buildings

        districtRects.forEach((dr, dIdx) => {{
            const distName = dr.item.name;
            const modules = packageGroups[distName];
            const shortName = distName.split('.').pop() || distName;

            // District ground plane
            const groundColor = districtColors[dIdx % districtColors.length];
            const groundGeo = new THREE.PlaneGeometry(dr.w * 0.95, dr.h * 0.95);
            const groundMat = new THREE.MeshStandardMaterial({{
                color: groundColor,
                roughness: 0.9,
                metalness: 0.1,
            }});
            const ground = new THREE.Mesh(groundGeo, groundMat);
            ground.rotation.x = -Math.PI / 2;
            ground.position.set(dr.x + dr.w / 2, 0.01, dr.y + dr.h / 2);
            ground.receiveShadow = true;
            scene.add(ground);

            // District border
            const borderGeo = new THREE.EdgesGeometry(new THREE.PlaneGeometry(dr.w * 0.96, dr.h * 0.96));
            const borderMat = new THREE.LineBasicMaterial({{ color: 0x333355, transparent: true, opacity: 0.5 }});
            const border = new THREE.LineSegments(borderGeo, borderMat);
            border.rotation.x = -Math.PI / 2;
            border.position.set(dr.x + dr.w / 2, 0.02, dr.y + dr.h / 2);
            scene.add(border);

            // District label (flag pole)
            const labelDiv = document.createElement('div');
            labelDiv.style.cssText = `
                color: #aaa; font-size: 11px; font-weight: 600;
                background: rgba(0,0,0,0.7); padding: 3px 8px; border-radius: 4px;
                border-left: 3px solid #${{groundColor.toString(16).padStart(6,'0').replace(/^(.)(.)(.)$/,'$1$1$2$2$3$3')}};
                white-space: nowrap; pointer-events: none;
            `;
            labelDiv.textContent = shortName + ` (${{modules.length}})`;
            const label = new CSS2DObject(labelDiv);
            label.position.set(dr.x + dr.w / 2, 0.5, dr.y + 1.5);
            scene.add(label);

            // Inner treemap for buildings within district
            const innerPad = dr.w * (1 - PADDING) / 2;
            const buildingItems = modules.map(m => ({{
                module: m,
                value: Math.max(m.lines_of_code || 1, 10)
            }}));

            const buildingRects = treemapLayout(
                buildingItems,
                dr.x + innerPad,
                dr.y + innerPad + 2, // offset for label
                dr.w * PADDING,
                dr.h * PADDING - 2
            );

            buildingRects.forEach(br => {{
                const m = br.item.module;

                // Height: cyclomatic complexity (log-scaled)
                const rawCC = m.total_cyclomatic || 1;
                const height = Math.max(0.5, Math.log10(rawCC + 1) * 4);

                // Footprint: from treemap rect, with small gap
                const bw = Math.max(0.3, br.w - GAP * 0.3);
                const bd = Math.max(0.3, br.h - GAP * 0.3);

                const geometry = new THREE.BoxGeometry(bw, height, bd);
                const healthColor = new THREE.Color(m.color);

                const material = new THREE.MeshStandardMaterial({{
                    color: healthColor,
                    roughness: 0.55,
                    metalness: 0.15,
                    emissive: healthColor,
                    emissiveIntensity: m.health < 0.35 ? 0.15 : 0.02,
                }});

                const building = new THREE.Mesh(geometry, material);
                building.position.set(
                    br.x + br.w / 2,
                    height / 2,
                    br.y + br.h / 2
                );
                building.castShadow = true;
                building.receiveShadow = true;
                building.userData = {{ ...m, _district: shortName }};
                scene.add(building);
                buildings.push(building);
            }});

            districtGroups.push({{ name: distName, shortName, rect: dr, moduleCount: modules.length }});
        }});

        // ====================================================================
        // Camera: OrbitControls-style with proper panning
        // ====================================================================
        let isDragging = false;
        let isPanning = false;
        let previousMouse = {{ x: 0, y: 0 }};
        let spherical = {{ radius: citySize * 0.8, theta: Math.PI / 4, phi: Math.PI / 4 }};
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
                spherical.theta -= deltaX * 0.008;
                spherical.phi -= deltaY * 0.008;
                spherical.phi = Math.max(0.1, Math.min(Math.PI / 2 - 0.05, spherical.phi));
                updateCamera();
            }}
            if (isPanning) {{
                // Proper screen-space panning: move target along camera's right and up vectors
                const panSpeed = spherical.radius * 0.001;
                const camRight = new THREE.Vector3();
                const camUp = new THREE.Vector3();
                camera.matrixWorld.extractBasis(camRight, camUp, new THREE.Vector3());
                // Project right/up onto ground plane (XZ)
                camRight.y = 0;
                camRight.normalize();
                const groundForward = new THREE.Vector3();
                camera.getWorldDirection(groundForward);
                groundForward.y = 0;
                groundForward.normalize();

                target.add(camRight.multiplyScalar(-deltaX * panSpeed));
                target.add(groundForward.multiplyScalar(deltaY * panSpeed));
                updateCamera();
            }}
            previousMouse = {{ x: e.clientX, y: e.clientY }};
        }});

        window.addEventListener('mouseup', () => {{ isDragging = false; isPanning = false; }});
        renderer.domElement.addEventListener('contextmenu', e => e.preventDefault());

        renderer.domElement.addEventListener('wheel', e => {{
            e.preventDefault();
            spherical.radius *= 1 + e.deltaY * 0.001;
            spherical.radius = Math.max(5, Math.min(citySize * 2, spherical.radius));
            updateCamera();
        }}, {{ passive: false }});

        // ====================================================================
        // Raycasting for Tooltips + Selection
        // ====================================================================
        const raycaster = new THREE.Raycaster();
        const mouse = new THREE.Vector2();
        const tooltip = document.getElementById('tooltip');
        let selectedBuilding = null;

        function healthBreakdown(m) {{
            // Replicate the health calculation to show breakdown
            const avgCC = m.function_count > 0 ? m.total_cyclomatic / m.function_count : 0;
            const avgCog = m.function_count > 0 ? m.total_cognitive / m.function_count : 0;
            const locPerFunc = m.function_count > 0 ? m.lines_of_code / m.function_count : 0;
            const totalCoupling = m.ca + m.ce;

            const ccScore = Math.max(0, Math.min(1, 1 - Math.max(0, (avgCC - 5) / 15)));
            const cogScore = Math.max(0, Math.min(1, 1 - avgCog / 30));
            const locScore = Math.max(0, Math.min(1, 1 - Math.max(0, (locPerFunc - 50) / 100)));
            const couplingScore = totalCoupling === 0 ? 0.6 : Math.max(0, Math.min(1, 1 - (totalCoupling - 10) / 30));

            let sizeScore;
            if (m.function_count < 2) sizeScore = 0.4;
            else if (m.function_count <= 30) sizeScore = 1.0;
            else if (m.function_count <= 50) sizeScore = 0.7;
            else sizeScore = Math.max(0.2, 1 - (m.function_count - 50) / 100);

            function scoreColor(s) {{
                if (s >= 0.8) return '#00ff88';
                if (s >= 0.5) return '#88cc55';
                if (s >= 0.35) return '#ddaa33';
                return '#ff4444';
            }}

            return `
                <div class="breakdown-item"><span>Complexity (avg CC ${{avgCC.toFixed(1)}})</span><span class="score" style="color:${{scoreColor(ccScore)}}">${{(ccScore*100).toFixed(0)}}%</span></div>
                <div class="breakdown-item"><span>Cognitive (avg ${{avgCog.toFixed(1)}})</span><span class="score" style="color:${{scoreColor(cogScore)}}">${{(cogScore*100).toFixed(0)}}%</span></div>
                <div class="breakdown-item"><span>LOC/func (${{locPerFunc.toFixed(0)}})</span><span class="score" style="color:${{scoreColor(locScore)}}">${{(locScore*100).toFixed(0)}}%</span></div>
                <div class="breakdown-item"><span>Coupling (Ca:${{m.ca}} Ce:${{m.ce}})</span><span class="score" style="color:${{scoreColor(couplingScore)}}">${{(couplingScore*100).toFixed(0)}}%</span></div>
                <div class="breakdown-item"><span>Module size (${{m.function_count}} funcs)</span><span class="score" style="color:${{scoreColor(sizeScore)}}">${{(sizeScore*100).toFixed(0)}}%</span></div>
            `;
        }}

        renderer.domElement.addEventListener('mousemove', e => {{
            mouse.x = (e.clientX / window.innerWidth) * 2 - 1;
            mouse.y = -(e.clientY / window.innerHeight) * 2 + 1;

            raycaster.setFromCamera(mouse, camera);
            const intersects = raycaster.intersectObjects(buildings);

            if (intersects.length > 0) {{
                const m = intersects[0].object.userData;
                document.body.style.cursor = 'pointer';
                tooltip.style.display = 'block';

                let left = e.clientX + 15;
                let top = e.clientY + 15;
                if (left + 360 > window.innerWidth) left = e.clientX - 375;
                if (top + 300 > window.innerHeight) top = e.clientY - 315;

                tooltip.style.left = left + 'px';
                tooltip.style.top = top + 'px';
                tooltip.innerHTML = `
                    <h3>${{m.name}}</h3>
                    <div class="section-label">Location</div>
                    <div class="metric"><span class="label">District</span><span class="value">${{m._district}}</span></div>
                    <div class="metric"><span class="label">Slice</span><span class="value">${{m.slice}}</span></div>
                    <div class="metric"><span class="label">Layer</span><span class="value">${{m.layer}}</span></div>
                    <div class="section-label">Metrics</div>
                    <div class="metric"><span class="label">Lines of Code</span><span class="value">${{m.lines_of_code.toLocaleString()}}</span></div>
                    <div class="metric"><span class="label">Functions</span><span class="value">${{m.function_count}}</span></div>
                    <div class="metric"><span class="label">Cyclomatic</span><span class="value">${{m.total_cyclomatic}}</span></div>
                    <div class="metric"><span class="label">Cognitive</span><span class="value">${{m.total_cognitive}}</span></div>
                    <div class="section-label">Health <span style="color:${{m.color}}; font-weight:600">${{(m.health * 100).toFixed(0)}}% ${{m.health_label}}</span></div>
                    <div class="health-bar"><div class="health-fill" style="width:${{m.health*100}}%; background:${{m.color}}"></div></div>
                    <div class="health-breakdown">${{healthBreakdown(m)}}</div>
                `;

                // Highlight on hover
                if (!selectedBuilding) {{
                    intersects[0].object.material.emissiveIntensity = 0.4;
                }}
            }} else {{
                tooltip.style.display = 'none';
                document.body.style.cursor = 'default';
                if (!selectedBuilding) {{
                    buildings.forEach(b => {{
                        b.material.emissiveIntensity = b.userData.health < 0.35 ? 0.15 : 0.02;
                    }});
                }}
            }}
        }});

        // Click to select/deselect
        renderer.domElement.addEventListener('click', e => {{
            raycaster.setFromCamera(mouse, camera);
            const intersects = raycaster.intersectObjects(buildings);

            if (intersects.length > 0) {{
                const clicked = intersects[0].object;
                if (selectedBuilding === clicked) {{
                    // Deselect
                    selectedBuilding = null;
                    buildings.forEach(b => {{
                        b.material.emissiveIntensity = b.userData.health < 0.35 ? 0.15 : 0.02;
                        b.material.opacity = 1.0;
                        b.material.transparent = false;
                    }});
                }} else {{
                    // Select: highlight same-district buildings
                    selectedBuilding = clicked;
                    const district = clicked.userData._district;
                    buildings.forEach(b => {{
                        if (b.userData._district === district) {{
                            b.material.emissiveIntensity = 0.2;
                            b.material.opacity = 1.0;
                            b.material.transparent = false;
                        }} else {{
                            b.material.emissiveIntensity = 0.01;
                            b.material.opacity = 0.3;
                            b.material.transparent = true;
                        }}
                    }});
                    clicked.material.emissiveIntensity = 0.5;
                }}
            }} else {{
                // Click empty: deselect
                selectedBuilding = null;
                buildings.forEach(b => {{
                    b.material.emissiveIntensity = b.userData.health < 0.35 ? 0.15 : 0.02;
                    b.material.opacity = 1.0;
                    b.material.transparent = false;
                }});
            }}
        }});

        // ====================================================================
        // Minimap
        // ====================================================================
        const minimapCanvas = document.getElementById('minimap-canvas');
        const mmCtx = minimapCanvas.getContext('2d');
        minimapCanvas.width = 180;
        minimapCanvas.height = 180;

        function drawMinimap() {{
            mmCtx.fillStyle = '#0a0a0f';
            mmCtx.fillRect(0, 0, 180, 180);

            const scale = 160 / citySize;
            const ox = 90, oy = 90;

            // Draw districts
            districtRects.forEach((dr, i) => {{
                mmCtx.fillStyle = '#' + districtColors[i % districtColors.length].toString(16).padStart(6, '0');
                mmCtx.fillRect(
                    ox + dr.x * scale,
                    oy + dr.y * scale,
                    dr.w * scale,
                    dr.h * scale
                );
            }});

            // Draw camera position
            mmCtx.fillStyle = '#ff4444';
            mmCtx.beginPath();
            mmCtx.arc(ox + target.x * scale, oy + target.z * scale, 3, 0, Math.PI * 2);
            mmCtx.fill();

            // Draw camera FOV cone
            mmCtx.strokeStyle = 'rgba(255,100,100,0.4)';
            mmCtx.lineWidth = 1;
            const camDir = new THREE.Vector3();
            camera.getWorldDirection(camDir);
            const angle = Math.atan2(camDir.z, camDir.x);
            const fovRad = 0.5;
            mmCtx.beginPath();
            mmCtx.moveTo(ox + target.x * scale, oy + target.z * scale);
            mmCtx.lineTo(
                ox + (target.x + Math.cos(angle - fovRad) * 30) * scale,
                oy + (target.z + Math.sin(angle - fovRad) * 30) * scale
            );
            mmCtx.moveTo(ox + target.x * scale, oy + target.z * scale);
            mmCtx.lineTo(
                ox + (target.x + Math.cos(angle + fovRad) * 30) * scale,
                oy + (target.z + Math.sin(angle + fovRad) * 30) * scale
            );
            mmCtx.stroke();
        }}

        // ====================================================================
        // Resize + Animation
        // ====================================================================
        window.addEventListener('resize', () => {{
            camera.aspect = window.innerWidth / window.innerHeight;
            camera.updateProjectionMatrix();
            renderer.setSize(window.innerWidth, window.innerHeight);
            labelRenderer.setSize(window.innerWidth, window.innerHeight);
        }});

        function animate() {{
            requestAnimationFrame(animate);
            renderer.render(scene, camera);
            labelRenderer.render(scene, camera);
            drawMinimap();
        }}
        animate();
    </script>
</body>
</html>"##,
        modules_json = modules_escaped,
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
