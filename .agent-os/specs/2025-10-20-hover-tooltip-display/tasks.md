# Spec Tasks

These are the tasks to be completed for the spec detailed in @.agent-os/specs/2025-10-20-hover-tooltip-display/spec.md

> Created: 2025-10-20
> Status: Ready for Implementation

## Tasks

- [x] 1. Implement Three.js Raycasting and Mouse Event Handling
  - [x] 1.1 Write tests for raycasting point detection with known positions
  - [x] 1.2 Add mousemove and mouseout event listeners to canvas in VectorVisualizationView.ts
  - [x] 1.3 Implement NDC coordinate conversion from screen coordinates
  - [x] 1.4 Create THREE.Raycaster instance and implement point intersection detection
  - [x] 1.5 Extract note data from intersected point index
  - [x] 1.6 Clean up event listeners in onClose() method
  - [x] 1.7 Verify all tests pass

- [ ] 2. Create Extensible Tooltip Data Structure
  - [ ] 2.1 Write tests for TooltipData interface and factory function
  - [ ] 2.2 Define TooltipData interface with noteName field and commented future fields
  - [ ] 2.3 Implement buildTooltipData() factory function in VectorDataManager.ts
  - [ ] 2.4 Add note metadata access by point index if not already available
  - [ ] 2.5 Verify all tests pass

- [ ] 3. Build Tooltip UI Component with Obsidian Theme Styling
  - [ ] 3.1 Write tests for tooltip DOM creation and theme class application
  - [ ] 3.2 Create tooltip DOM element (div) in VectorVisualizationView.ts
  - [ ] 3.3 Apply Obsidian theme CSS classes (tooltip/mod-tooltip) with fallback to CSS variables
  - [ ] 3.4 Style tooltip with proper padding, border-radius, shadow, and z-index
  - [ ] 3.5 Set pointer-events: none to avoid canvas interference
  - [ ] 3.6 Implement show/hide logic based on raycasting results
  - [ ] 3.7 Verify all tests pass

- [ ] 4. Implement Smart Positioning Algorithm
  - [ ] 4.1 Write tests for boundary positioning with deterministic viewport dimensions
  - [ ] 4.2 Implement calculateTooltipPosition() function with default offset logic
  - [ ] 4.3 Add viewport boundary detection (width and height checks)
  - [ ] 4.4 Implement horizontal flip (left of cursor) when exceeding right edge
  - [ ] 4.5 Implement vertical flip (above cursor) when exceeding bottom edge
  - [ ] 4.6 Handle corner cases requiring both flips
  - [ ] 4.7 Verify all tests pass

- [ ] 5. Integration Testing and Performance Validation
  - [ ] 5.1 Write integration test for complete hover workflow
  - [ ] 5.2 Test tooltip display responsiveness (target: <50ms)
  - [ ] 5.3 Verify light/dark theme adaptation
  - [ ] 5.4 Test edge case handling (no intersection, missing metadata)
  - [ ] 5.5 Manual testing in Obsidian with various vault sizes
  - [ ] 5.6 Run bun run lint and ensure passing
  - [ ] 5.7 Verify all tests pass
