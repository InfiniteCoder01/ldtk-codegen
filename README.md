# LDtk Code Gen
Generate typed rust code from LDtk Project, just like Haxe API (almost...)!

# Installation:
Just use cargo install or cargo binstall:<br />
`cargo install ldtk-codegen`

# Examples:
Generate demo.rs from project demo.ldtk using raylib Vector2 and Color:<br />
`ldtk-codegen demo.ldtk -v 'raylib::prelude::Vector2' -c 'raylib::prelude::Color'`

Generate project.rs from project demo.ldtk with serde support and preserving case:<br />
`ldtk-codegen demo.ldtk -o project.rs -p -s`
