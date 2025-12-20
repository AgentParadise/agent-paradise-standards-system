//! 3D Force-Directed Visualization
//!
//! Interactive 3D graph where nodes represent modules and edges represent coupling.
//! Uses Three.js for WebGL rendering with OrbitControls.
//!
//! ## Features
//! - Force-directed layout clusters coupled modules together
//! - Node color based on health (distance from main sequence)
//! - Edge thickness based on coupling strength
//! - Interactive sidebar with module list and coupling filter
//! - Click to focus, hover for tooltips

/// Generate 3D Force-Directed HTML visualization.
///
/// This function generates a complete, self-contained HTML document with
/// embedded Three.js for 3D rendering. The scene data is embedded as JSON.
///
/// # Arguments
/// * `scene_json` - Serialized Scene3D data (nodes, edges, camera)
/// * `node_count` - Number of nodes (for info panel)
/// * `edge_count` - Number of edges (for info panel)
///
/// # Returns
/// Complete HTML document as a string
#[allow(clippy::uninlined_format_args)]
pub fn generate(scene_json: &str, node_count: usize, edge_count: usize) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Code Topology - 3D Coupling Visualization</title>
    <style>
        body {{ margin: 0; overflow: hidden; font-family: 'SF Mono', 'Monaco', 'Inconsolata', monospace; display: flex; background: #0f0f1a; }}
        #main {{ flex: 1; position: relative; }}
        #info {{
            position: absolute;
            top: 10px;
            left: 10px;
            padding: 15px 20px;
            background: linear-gradient(135deg, rgba(20,20,40,0.95), rgba(40,30,60,0.9));
            color: #e0e0e0;
            border-radius: 12px;
            font-size: 13px;
            z-index: 100;
            border: 1px solid rgba(255,255,255,0.1);
            backdrop-filter: blur(10px);
            box-shadow: 0 8px 32px rgba(0,0,0,0.4);
        }}
        #info h3 {{ 
            margin: 0 0 12px 0; 
            font-size: 18px;
            background: linear-gradient(90deg, #ff6b9d, #c44569);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            font-weight: 700;
        }}
        #info p {{ margin: 4px 0; color: #b0b0b0; }}
        #info em {{ color: #888; font-size: 11px; }}
        #legend {{
            position: absolute;
            bottom: 20px;
            left: 10px;
            padding: 15px 20px;
            background: linear-gradient(135deg, rgba(20,20,40,0.95), rgba(40,30,60,0.9));
            color: #e0e0e0;
            border-radius: 12px;
            font-size: 12px;
            z-index: 100;
            border: 1px solid rgba(255,255,255,0.1);
        }}
        #legend h4 {{ margin: 0 0 10px 0; color: #aaa; font-weight: 500; }}
        .legend-item {{ display: flex; align-items: center; margin: 6px 0; }}
        .legend-color {{ width: 14px; height: 14px; border-radius: 50%; margin-right: 10px; }}
        #filter {{
            margin-top: 12px;
            padding-top: 12px;
            border-top: 1px solid rgba(255,255,255,0.1);
        }}
        #filter label {{ display: block; font-size: 11px; color: #888; margin-bottom: 6px; }}
        #filter input[type="range"] {{ width: 100%; cursor: pointer; }}
        #filter-value {{ float: right; color: #ff6b9d; }}
        #sidebar {{
            width: 280px;
            height: 100vh;
            background: linear-gradient(180deg, rgba(15,15,30,0.98), rgba(20,20,35,0.98));
            border-left: 1px solid rgba(255,255,255,0.1);
            overflow-y: scroll;
            padding: 15px;
            z-index: 100;
            box-sizing: border-box;
        }}
        #module-list {{
            max-height: calc(100vh - 80px);
            overflow-y: auto;
        }}
        #sidebar h2 {{ 
            font-size: 14px; 
            color: #888; 
            margin-bottom: 15px; 
            display: flex; 
            justify-content: space-between;
            align-items: center;
        }}
        #sidebar h2 span {{ color: #666; font-weight: normal; }}
        .module-item {{
            padding: 10px 12px;
            margin-bottom: 6px;
            background: rgba(255,255,255,0.03);
            border-radius: 8px;
            cursor: pointer;
            transition: all 0.2s;
            border: 1px solid transparent;
        }}
        .module-item:hover {{ border-color: rgba(255,255,255,0.1); background: rgba(255,255,255,0.06); }}
        .module-item.selected {{ border-color: #ff6b9d; background: rgba(255,107,157,0.1); }}
        .module-item.filtered-out {{ opacity: 0.3; pointer-events: none; }}
        .module-name {{ 
            font-weight: 500; 
            margin-bottom: 4px; 
            font-size: 12px; 
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }}
        .module-stats {{ font-size: 10px; color: #666; display: flex; gap: 10px; }}
        .module-coupling {{ color: #88aaff; }}
        .module-complexity {{ color: #ffaa88; }}
        #tooltip {{
            position: fixed;
            padding: 16px 20px;
            background: rgba(20,20,35,0.98);
            color: #fff;
            border-radius: 12px;
            font-size: 13px;
            pointer-events: none;
            display: none;
            z-index: 10000;
            border: 2px solid rgba(100,180,255,0.6);
            min-width: 220px;
            max-width: 320px;
            box-shadow: 0 8px 32px rgba(0,0,0,0.7), 0 0 20px rgba(100,180,255,0.3);
            backdrop-filter: blur(10px);
        }}
        #tooltip .name {{
            font-weight: 700;
            font-size: 16px;
            margin-bottom: 12px;
            color: #6bf;
            word-break: break-word;
        }}
        #tooltip .metric {{
            display: flex;
            justify-content: space-between;
            margin: 6px 0;
            border-bottom: 1px solid rgba(255,255,255,0.15);
            padding-bottom: 6px;
        }}
        #tooltip .metric-label {{ color: #aaa; font-size: 12px; }}
        #tooltip .metric-value {{ color: #fff; font-weight: 600; font-size: 14px; }}
        #tooltip .active-hint {{
            margin-top: 12px;
            padding-top: 10px;
            border-top: 1px solid rgba(100,180,255,0.4);
            font-size: 12px;
            color: #8cf;
            text-align: center;
            font-style: italic;
        }}
        .node-label {{
            color: #fff;
            font-size: 12px;
            font-weight: 600;
            text-shadow: 0 2px 8px rgba(0,0,0,0.8), 0 0 20px rgba(0,0,0,0.6);
            pointer-events: none;
            white-space: nowrap;
        }}
        .edge-label {{
            color: rgba(255,255,255,0.6);
            font-size: 10px;
            text-shadow: 0 1px 4px rgba(0,0,0,0.9);
            pointer-events: none;
        }}
    </style>
</head>
<body>
    <div id="main">
        <div id="info">
            <h3>🌐 Code Topology</h3>
            <p><strong>{node_count}</strong> modules</p>
            <p><strong>{edge_count}</strong> coupling relationships</p>
            <p><em>Drag to rotate • Scroll to zoom • Hover for details</em></p>
            <div id="filter">
                <label>Coupling Threshold <span id="filter-value">0%</span></label>
                <input type="range" id="coupling-slider" min="0" max="100" value="0">
            </div>
        </div>
        <div id="legend">
            <h4>Module Health</h4>
            <div class="legend-item"><div class="legend-color" style="background: #00cc88;"></div>🟢 Healthy (on main sequence)</div>
            <div class="legend-item"><div class="legend-color" style="background: #ffaa40;"></div>🟡 Moderate concern</div>
            <div class="legend-item"><div class="legend-color" style="background: #ff4040;"></div>🔴 Needs attention (Zone of Pain)</div>
            <h4 style="margin-top: 12px;">Coupling Strength</h4>
            <div class="legend-item"><div class="legend-color" style="background: #ffffff; border: 1px solid #666;"></div>Strong (≥0.7)</div>
            <div class="legend-item"><div class="legend-color" style="background: #888888;"></div>Medium (0.3-0.7)</div>
            <div class="legend-item"><div class="legend-color" style="background: #444444;"></div>Weak (&lt;0.3)</div>
        </div>
        <div id="tooltip"></div>
    </div>
    <div id="sidebar">
        <h2>Modules <span id="module-count"></span></h2>
        <div id="module-list"></div>
    </div>
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
        import {{ OrbitControls }} from 'three/addons/controls/OrbitControls.js';
        import {{ CSS2DRenderer, CSS2DObject }} from 'three/addons/renderers/CSS2DRenderer.js';
        
        const scene = new THREE.Scene();
        scene.background = new THREE.Color(0x0f0f1a);
        
        const mainContainer = document.getElementById('main');
        const mainWidth = mainContainer.clientWidth;
        
        const camera = new THREE.PerspectiveCamera(75, mainWidth / window.innerHeight, 0.1, 1000);
        camera.position.set(0, 5, 10);
        
        const renderer = new THREE.WebGLRenderer({{ antialias: true }});
        renderer.setSize(mainWidth, window.innerHeight);
        renderer.setPixelRatio(window.devicePixelRatio);
        mainContainer.appendChild(renderer.domElement);
        
        const labelRenderer = new CSS2DRenderer();
        labelRenderer.setSize(mainWidth, window.innerHeight);
        labelRenderer.domElement.style.position = 'absolute';
        labelRenderer.domElement.style.top = '0px';
        labelRenderer.domElement.style.pointerEvents = 'none';
        mainContainer.appendChild(labelRenderer.domElement);
        
        const controls = new OrbitControls(camera, renderer.domElement);
        controls.enableDamping = true;
        controls.dampingFactor = 0.05;
        
        scene.add(new THREE.AmbientLight(0xffffff, 0.4));
        const dirLight = new THREE.DirectionalLight(0xffffff, 0.8);
        dirLight.position.set(5, 10, 7);
        scene.add(dirLight);
        const backLight = new THREE.DirectionalLight(0x6060ff, 0.3);
        backLight.position.set(-5, -5, -5);
        scene.add(backLight);
        
        const data = {scene_json};
        
        const nodeMeshes = [];
        const tooltip = document.getElementById('tooltip');
        const raycaster = new THREE.Raycaster();
        const mouse = new THREE.Vector2();
        
        data.nodes.forEach(node => {{
            const nodeRadius = Math.max(0.6, node.size * 0.5);
            const geometry = new THREE.SphereGeometry(nodeRadius, 32, 32);
            const material = new THREE.MeshPhongMaterial({{
                color: node.color,
                emissive: node.color,
                emissiveIntensity: 0.4,
                shininess: 100,
                transparent: true,
                opacity: 1.0
            }});
            const mesh = new THREE.Mesh(geometry, material);
            mesh.position.set(...node.position);
            mesh.userData = node;
            scene.add(mesh);
            nodeMeshes.push(mesh);
            
            const labelDiv = document.createElement('div');
            labelDiv.className = 'node-label';
            labelDiv.textContent = node.label;
            const label = new CSS2DObject(labelDiv);
            label.position.set(0, node.size * 0.5 + 0.3, 0);
            mesh.add(label);
        }});
        
        const edgeMeshes = [];
        data.edges.forEach((edge, edgeIndex) => {{
            const fromNode = data.nodes.find(n => n.id === edge.from);
            const toNode = data.nodes.find(n => n.id === edge.to);
            if (fromNode && toNode) {{
                const start = new THREE.Vector3(...fromNode.position);
                const end = new THREE.Vector3(...toNode.position);
                const path = new THREE.LineCurve3(start, end);
                const tubeGeometry = new THREE.TubeGeometry(path, 1, edge.strength * 0.08 + 0.02, 8, false);
                const tubeMaterial = new THREE.MeshBasicMaterial({{
                    color: edge.color,
                    opacity: 0.4 + edge.strength * 0.4,
                    transparent: true
                }});
                const tube = new THREE.Mesh(tubeGeometry, tubeMaterial);
                tube.userData = {{ edgeIndex, from: edge.from, to: edge.to, strength: edge.strength }};
                edgeMeshes.push(tube);
                scene.add(tube);
                
                if (edge.strength >= 0.5) {{
                    const midpoint = start.clone().add(end).multiplyScalar(0.5);
                    const edgeLabelDiv = document.createElement('div');
                    edgeLabelDiv.className = 'edge-label';
                    edgeLabelDiv.textContent = edge.strength.toFixed(2);
                    const edgeLabel = new CSS2DObject(edgeLabelDiv);
                    edgeLabel.position.copy(midpoint);
                    scene.add(edgeLabel);
                }}
            }}
        }});
        
        let hoveredModule = null;
        let hoverSource = null;
        let currentThreshold = 0;
        
        function highlightModule(moduleId, source) {{
            hoveredModule = moduleId;
            hoverSource = source || '3d';
            
            const connectedModules = new Set([moduleId]);
            data.edges.forEach(edge => {{
                if (edge.from === moduleId) connectedModules.add(edge.to);
                if (edge.to === moduleId) connectedModules.add(edge.from);
            }});
            
            nodeMeshes.forEach(mesh => {{
                const isConnected = connectedModules.has(mesh.userData.id);
                mesh.material.opacity = isConnected ? 1.0 : 0.15;
                mesh.material.transparent = true;
                mesh.scale.setScalar(mesh.userData.id === moduleId ? 1.4 : (isConnected ? 1.1 : 0.8));
            }});
            
            edgeMeshes.forEach(mesh => {{
                const isConnected = mesh.userData.from === moduleId || mesh.userData.to === moduleId;
                mesh.material.opacity = isConnected ? 1.0 : 0.05;
                mesh.material.transparent = true;
            }});
            
            document.querySelectorAll('.module-item').forEach(el => {{
                const isConnected = connectedModules.has(el.dataset.id);
                el.style.opacity = isConnected ? '1' : '0.3';
            }});
        }}
        
        function clearHighlight(source) {{
            if (!hoveredModule) return;
            if (source && hoverSource !== source) return;
            
            hoveredModule = null;
            hoverSource = null;
            
            nodeMeshes.forEach(mesh => {{
                mesh.material.opacity = 1.0;
                mesh.scale.setScalar(1.0);
            }});
            
            edgeMeshes.forEach(mesh => {{
                mesh.material.opacity = 0.6;
                mesh.visible = mesh.userData.strength >= currentThreshold;
            }});
            
            document.querySelectorAll('.module-item').forEach(el => {{
                el.style.opacity = '';
            }});
        }}
        
        let activeModule = null;
        let selectedModule = null;
        
        function onMouseMove(event) {{
            const rect = renderer.domElement.getBoundingClientRect();
            mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
            mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;
            
            raycaster.setFromCamera(mouse, camera);
            const intersects = raycaster.intersectObjects(nodeMeshes);
            
            if (intersects.length > 0) {{
                const node = intersects[0].object.userData;
                tooltip.style.display = 'block';
                
                const tooltipWidth = 280;
                const tooltipHeight = 200;
                let left = event.clientX + 20;
                let top = event.clientY + 20;
                
                if (left + tooltipWidth > window.innerWidth) {{
                    left = event.clientX - tooltipWidth - 20;
                }}
                if (top + tooltipHeight > window.innerHeight) {{
                    top = event.clientY - tooltipHeight - 20;
                }}
                
                tooltip.style.left = left + 'px';
                tooltip.style.top = top + 'px';
                document.body.style.cursor = 'pointer';
                
                const hoveredMesh = intersects[0].object;
                if (!hoveredMesh.userData.isHovered) {{
                    nodeMeshes.forEach(m => {{
                        if (m.userData.isHovered && m !== hoveredMesh) {{
                            m.userData.isHovered = false;
                            const isActive = m.userData.id === activeModule;
                            const isSelected = m.userData.id === selectedModule;
                            m.material.emissiveIntensity = isActive ? 0.6 : (isSelected ? 0.5 : 0.4);
                            m.scale.setScalar(isActive || isSelected ? 1.1 : 1.0);
                        }}
                    }});
                    hoveredMesh.userData.isHovered = true;
                    hoveredMesh.material.emissiveIntensity = 0.8;
                    hoveredMesh.scale.setScalar(1.15);
                }}

                const connections = data.edges
                    .filter(e => e.from === node.id || e.to === node.id)
                    .map(e => {{
                        const other = e.from === node.id ? e.to : e.from;
                        return `${{other}} (${{e.strength.toFixed(2)}})`;
                    }})
                    .join(', ');
                
                tooltip.innerHTML = `
                    <div class="name">${{node.label}}</div>
                    <div class="metric">
                        <span class="metric-label">Cyclomatic</span>
                        <span class="metric-value">${{node.metrics.cyclomatic}}</span>
                    </div>
                    <div class="metric">
                        <span class="metric-label">Cognitive</span>
                        <span class="metric-value">${{node.metrics.cognitive}}</span>
                    </div>
                    <div class="metric">
                        <span class="metric-label">Instability</span>
                        <span class="metric-value">${{(node.metrics.instability * 100).toFixed(0)}}%</span>
                    </div>
                    <div class="metric">
                        <span class="metric-label">Connected to</span>
                        <span class="metric-value">${{connections || 'none'}}</span>
                    </div>
                    ${{activeModule === node.id ? '<div class="active-hint">Click to deactivate</div>' : '<div class="active-hint">Click to focus connections</div>'}}
                `;
            }} else {{
                tooltip.style.display = 'none';
                document.body.style.cursor = 'default';
                
                nodeMeshes.forEach(m => {{
                    if (m.userData.isHovered) {{
                        m.userData.isHovered = false;
                        const isActive = m.userData.id === activeModule;
                        const isSelected = m.userData.id === selectedModule;
                        m.material.emissiveIntensity = isActive ? 0.6 : (isSelected ? 0.5 : 0.4);
                        m.scale.setScalar(isActive || isSelected ? 1.1 : 1.0);
                    }}
                }});
            }}
        }}

        function onClick(event) {{
            raycaster.setFromCamera(mouse, camera);
            const intersects = raycaster.intersectObjects(nodeMeshes);
            
            if (intersects.length > 0) {{
                const node = intersects[0].object.userData;
                
                if (activeModule === node.id) {{
                    activeModule = null;
                    clearHighlight('3d');
                }} else {{
                    activeModule = node.id;
                    highlightModule(node.id, '3d');
                }}
            }} else {{
                if (activeModule) {{
                    activeModule = null;
                    clearHighlight('3d');
                }}
            }}
        }}

        window.addEventListener('mousemove', onMouseMove);
        window.addEventListener('click', onClick);
        
        const moduleCoupling = {{}};
        data.nodes.forEach(n => moduleCoupling[n.id] = 0);
        data.edges.forEach(e => {{
            moduleCoupling[e.from] = (moduleCoupling[e.from] || 0) + e.strength;
            moduleCoupling[e.to] = (moduleCoupling[e.to] || 0) + e.strength;
        }});
        
        const sortedModules = [...data.nodes].sort((a, b) => 
            (moduleCoupling[b.id] || 0) - (moduleCoupling[a.id] || 0)
        );
        
        const sidebarList = document.getElementById('module-list');
        
        function renderSidebar() {{
            document.getElementById('module-count').textContent = `(${{data.nodes.length}})`;
            sidebarList.innerHTML = sortedModules.map(m => {{
                const coupling = moduleCoupling[m.id] || 0;
                const shortName = m.label.split('::').pop() || m.label;
                return `
                    <div class="module-item ${{selectedModule === m.id ? 'selected' : ''}}" data-id="${{m.id}}">
                        <div class="module-name" style="color:${{m.color}}">${{shortName}}</div>
                        <div class="module-stats">
                            <span class="module-coupling">⚡ ${{coupling.toFixed(1)}}</span>
                            <span class="module-complexity">📊 CC:${{m.metrics.cyclomatic}}</span>
                            <span>🔧 ${{m.metrics.function_count}}</span>
                        </div>
                    </div>
                `;
            }}).join('');
        }}
        
        sidebarList.addEventListener('click', (e) => {{
            const item = e.target.closest('.module-item');
            if (!item) return;
            
            const id = item.dataset.id;
            selectedModule = selectedModule === id ? null : id;
            renderSidebar();
            
            if (selectedModule) {{
                const node = data.nodes.find(n => n.id === selectedModule);
                if (node) {{
                    const targetPos = new THREE.Vector3(...node.position);
                    controls.target.copy(targetPos);
                    camera.position.set(
                        targetPos.x + 5,
                        targetPos.y + 3,
                        targetPos.z + 5
                    );
                }}
            }}
            
            nodeMeshes.forEach(mesh => {{
                const isSelected = mesh.userData.id === selectedModule;
                mesh.material.emissiveIntensity = isSelected ? 0.6 : 0.2;
                mesh.scale.setScalar(isSelected ? 1.3 : 1.0);
            }});
        }});
        
        sidebarList.addEventListener('mouseover', (e) => {{
            const item = e.target.closest('.module-item');
            if (!item) return;
            highlightModule(item.dataset.id, 'sidebar');
        }});
        
        sidebarList.addEventListener('mouseout', (e) => {{
            const item = e.target.closest('.module-item');
            if (!item) return;
            if (!e.relatedTarget || !e.relatedTarget.closest || e.relatedTarget.closest('.module-item') !== item) {{
                clearHighlight('sidebar');
            }}
        }});
        
        renderSidebar();
        
        document.getElementById('coupling-slider').addEventListener('input', e => {{
            currentThreshold = parseInt(e.target.value) / 100;
            document.getElementById('filter-value').textContent = `${{e.target.value}}%`;
            
            edgeMeshes.forEach(mesh => {{
                mesh.visible = mesh.userData.strength >= currentThreshold;
            }});
            
            const visibleModules = new Set();
            data.edges.forEach(edge => {{
                if (edge.strength >= currentThreshold) {{
                    visibleModules.add(edge.from);
                    visibleModules.add(edge.to);
                }}
            }});
            
            document.querySelectorAll('.module-item').forEach(item => {{
                const moduleId = item.dataset.id;
                const hasVisibleEdge = visibleModules.has(moduleId);
                item.classList.toggle('filtered-out', currentThreshold > 0 && !hasVisibleEdge);
            }});
            
            nodeMeshes.forEach(mesh => {{
                const hasVisibleEdge = visibleModules.has(mesh.userData.id);
                mesh.material.opacity = (currentThreshold > 0 && !hasVisibleEdge) ? 0.2 : 1.0;
                mesh.material.transparent = true;
            }});
        }});
        
        function animate() {{
            requestAnimationFrame(animate);
            controls.update();
            renderer.render(scene, camera);
            labelRenderer.render(scene, camera);
        }}
        animate();
        
        window.addEventListener('resize', () => {{
            const mainWidth = document.getElementById('main').clientWidth;
            camera.aspect = mainWidth / window.innerHeight;
            camera.updateProjectionMatrix();
            renderer.setSize(mainWidth, window.innerHeight);
            labelRenderer.setSize(mainWidth, window.innerHeight);
        }});
        
        window.dispatchEvent(new Event('resize'));
    </script>
</body>
</html>"#,
        node_count = node_count,
        edge_count = edge_count,
        scene_json = scene_json
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_contains_doctype() {
        let html = generate("{}", 0, 0);
        assert!(html.starts_with("<!DOCTYPE html>"));
    }

    #[test]
    fn test_generate_contains_title() {
        let html = generate("{}", 5, 3);
        assert!(html.contains("<title>Code Topology"));
    }

    #[test]
    fn test_generate_contains_counts() {
        let html = generate("{}", 42, 17);
        assert!(html.contains("42"));
        assert!(html.contains("17"));
    }
}
