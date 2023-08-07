# LDtk Code Gen
Generate typed rust code from LDtk Project, just like Haxe API (almost...)!

# Examples:
Generate demo.rs from project demo.ldtk supporting raylib Vector2 and Color:
`ldtk-codegen demo.ldtk -v 'raylib::prelude::Vector2' -c 'raylib::prelude::Color'`

Generate project.rs from project demo.ldtk supporting serde and preserving case:
`ldtk-codegen demo.ldtk -o project.rs -p -s`
