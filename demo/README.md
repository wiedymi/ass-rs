# ASS Subtitle Renderer - Web Demo

A modern, user-friendly web interface for rendering Advanced SubStation Alpha (ASS/SSA) subtitles using WebAssembly.

## Features

- 🎨 **Modern UI**: Beautiful, responsive interface with drag-and-drop support
- 🚀 **WebAssembly Powered**: Fast subtitle rendering using Rust-compiled WASM
- 📱 **Mobile Friendly**: Responsive design that works on all devices
- 🔧 **Real-time Controls**: Adjust time, dimensions, and font size in real-time
- 💾 **Export Options**: Download rendered frames as PNG images
- 🔄 **ASS Normalization**: Built-in ASS file normalization tool

## How to Use

1. **Open the Demo**: Open `index.html` in a modern web browser
2. **Load Files**: 
   - Drag and drop or click to select an ASS/SSA subtitle file
   - Drag and drop or click to select a font file (TTF/OTF)
3. **Adjust Settings**:
   - Use the time slider to navigate through the subtitle timeline
   - Adjust width, height, and font size as needed
4. **Render**: Click "Render Frame" to generate the subtitle image
5. **Download**: Click "Download Image" to save the rendered frame

## Supported File Types

- **Subtitles**: `.ass`, `.ssa` (Advanced SubStation Alpha)
- **Fonts**: `.ttf`, `.otf`, `.woff`, `.woff2`

## Browser Requirements

- Modern browser with WebAssembly support (Chrome 57+, Firefox 52+, Safari 11+, Edge 16+)
- JavaScript enabled

## Additional Features

- **Normalize ASS**: Clean and standardize ASS subtitle files
- **Real-time Preview**: See changes immediately as you adjust settings
- **Error Handling**: Clear error messages and status updates
- **File Information**: View loaded file details and sizes

## Technical Details

This demo uses:
- **WebAssembly**: For high-performance subtitle rendering
- **Canvas API**: For displaying rendered frames
- **Modern JavaScript**: ES6 modules and async/await
- **Responsive CSS**: Grid and flexbox layouts

The WASM module is prebuilt and included in the `pkg/` directory, so no additional build steps are required.