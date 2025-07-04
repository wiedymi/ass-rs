// Web Worker for ASS subtitle rendering
// This runs in a separate thread for better performance

import init, { RendererHandle, normalize_ass } from './pkg/ass_wasm.js';

let wasmInitialized = false;
let rendererHandle = null;

// Initialize WASM module
async function initWasm() {
    if (!wasmInitialized) {
        await init();
        wasmInitialized = true;
        console.log('WASM initialized in worker');
    }
}

// Handle messages from main thread
self.onmessage = async function(e) {
    const { type, data, id } = e.data;
    
    try {
        await initWasm();
        
        switch (type) {
            case 'create_renderer':
                const { assContent, fontData, width, height } = data;
                rendererHandle = new RendererHandle(assContent, fontData, width, height);
                self.postMessage({ 
                    type: 'renderer_created', 
                    id,
                    success: true 
                });
                break;
                
            case 'render_frame':
                if (!rendererHandle) {
                    throw new Error('Renderer not initialized');
                }
                const { time } = data;
                const imageData = rendererHandle.render_frame(time);
                
                // Transfer the image data efficiently
                self.postMessage({ 
                    type: 'frame_rendered', 
                    id,
                    imageData: imageData,
                    success: true 
                }, [imageData.buffer]);
                break;
                
            case 'normalize_ass':
                const { content } = data;
                const normalized = normalize_ass(content);
                self.postMessage({ 
                    type: 'ass_normalized', 
                    id,
                    normalized,
                    success: true 
                });
                break;
                
            case 'update_dimensions':
                if (!rendererHandle) {
                    throw new Error('Renderer not initialized');
                }
                const { newWidth, newHeight } = data;
                rendererHandle.update_dimensions(newWidth, newHeight);
                self.postMessage({ 
                    type: 'dimensions_updated', 
                    id,
                    success: true 
                });
                break;
                
            default:
                throw new Error(`Unknown message type: ${type}`);
        }
    } catch (error) {
        console.error('Worker error:', error);
        self.postMessage({ 
            type: 'error', 
            id,
            error: error.message,
            success: false 
        });
    }
};

// Handle worker errors
self.onerror = function(error) {
    console.error('Worker script error:', error);
    self.postMessage({ 
        type: 'error', 
        error: error.message,
        success: false 
    });
};