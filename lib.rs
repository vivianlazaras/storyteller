
pub enum Layouts {
    Dot,
    Neato
}


/*
{
    unsafe {
        let dot = CString::new("digraph G { a -> b; b -> c; }").unwrap();
        let gvc = gvContext();
        // Parse DOT into graph
        let graph = agmemread(dot.as_ptr());
        if graph.is_null() {
            panic!("Failed to parse DOT");
        }

        // Layout graph with "dot"
        if gvLayout(gvc, graph, CString::new("dot").unwrap().as_ptr()) != 0 {
            panic!("Layout failed");
        }

        // Render to SVG in memory
        let mut result_ptr: *mut ::std::os::raw::c_char = ptr::null_mut();
        let mut length: usize = 0;

        if gvRenderData(
            gvc,
            graph,
            CString::new("svg").unwrap().as_ptr(),
            &mut result_ptr,
            &mut length,
        ) != 0
        {
            panic!("Render failed");
        }

        // Convert to Rust string
        let svg_slice = std::slice::from_raw_parts(result_ptr as *const u8, length as usize);
        let svg = String::from_utf8_lossy(svg_slice);
        println!("{}", svg);

        // Clean up
        gvFreeRenderData(result_ptr);
        gvFreeLayout(gvc, graph);
        agclose(graph);
        gvFreeContext(gvc);
    }
}
*/