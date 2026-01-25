+++
title = "Bumpers & Branding"
description = "Create professional station idents and transitions."
weight = 3
+++

Slatron includes a powerful bumper system for creating professional station branding elements like idents, transitions, and lower thirds.

## Bumper Architecture

1. **Bumper Backs**: Base video files that provide background animation or graphics.
2. **Bumper Templates**: MLT templates that overlay station branding on top of bumper backs.

## Default Bumpers
Slatron ships with:
- **3 Simple Bumper Backs**: Solid Blue, Solid Purple, Solid Grey.
- **1 Station Ident Template**: Displays station name with theme color outline.

## Custom MLT Example
```xml
<?xml version="1.0"?>
<mlt LC_NUMERIC="C" version="7.0.0">
  <profile description="HD 1080p 30 fps" width="1920" height="1080"/>
  <producer id="back" mlt_service="avformat">
    <property name="resource">{{BUMPER_BACK_PATH}}</property>
  </producer>
  <producer id="text" mlt_service="pango">
    <property name="markup">{{STATION_NAME}}</property>
    <property name="fgcolour">#ffffff</property>
    <property name="olcolour">{{THEME_COLOR}}</property>
  </producer>
  <tractor id="tractor0" in="0" out="149">
    <track producer="back"/>
    <track producer="text"/>
  </tractor>
</mlt>
```
