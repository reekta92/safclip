pub struct TimelineState {
    pub zoom_level: f64,
    pub pan_offset: f64,
    pub duration: f64,
}

impl TimelineState {
    pub fn new(duration: f64) -> Self {
        Self {
            zoom_level: 1.0,
            pan_offset: 0.0,
            duration,
        }
    }

    pub fn pixel_to_time(&self, pixel_x: u16, timeline_width: u16) -> f64 {
        if timeline_width == 0 {
            return self.pan_offset;
        }
        let visible_duration = self.duration / self.zoom_level;
        self.pan_offset + (pixel_x as f64 / timeline_width as f64) * visible_duration
    }

    pub fn time_to_pixel(&self, time: f64, timeline_width: u16) -> u16 {
        if timeline_width == 0 {
            return 0;
        }
        let visible_duration = self.duration / self.zoom_level;
        let px = ((time - self.pan_offset) / visible_duration) * timeline_width as f64;
        px.round() as u16
    }

    pub fn zoom_in(&mut self, factor: f64, anchor_time: f64) {
        let old_zoom = self.zoom_level;
        self.zoom_level *= factor;
        // Adjust pan_offset to keep anchor_time at the same relative position
        self.pan_offset = anchor_time - (anchor_time - self.pan_offset) * (old_zoom / self.zoom_level);
        self.clamp();
    }

    pub fn zoom_out(&mut self, factor: f64, anchor_time: f64) {
        let old_zoom = self.zoom_level;
        self.zoom_level /= factor;
        if self.zoom_level < 1.0 {
            self.zoom_level = 1.0;
        }
        // Adjust pan_offset to keep anchor_time at the same relative position
        self.pan_offset = anchor_time - (anchor_time - self.pan_offset) * (old_zoom / self.zoom_level);
        self.clamp();
    }

    pub fn pan(&mut self, delta_pixels: i16, timeline_width: u16) {
        if timeline_width == 0 {
            return;
        }
        let visible_duration = self.duration / self.zoom_level;
        let time_delta = (delta_pixels as f64 / timeline_width as f64) * visible_duration;
        self.pan_offset += time_delta;
        self.clamp();
    }

    pub fn visible_range(&self, timeline_width: u16) -> (f64, f64) {
        let start = self.pixel_to_time(0, timeline_width);
        let end = self.pixel_to_time(timeline_width, timeline_width);
        (start, end)
    }

    fn clamp(&mut self) {
        let visible_duration = self.duration / self.zoom_level;
        if self.zoom_level < 1.0 {
            self.zoom_level = 1.0;
        }
        
        if self.pan_offset < 0.0 {
            self.pan_offset = 0.0;
        }
        
        let max_pan = (self.duration - visible_duration).max(0.0);
        if self.pan_offset > max_pan {
            self.pan_offset = max_pan;
        }
    }
}
