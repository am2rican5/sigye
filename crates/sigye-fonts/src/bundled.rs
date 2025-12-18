//! Bundled FIGlet fonts embedded in the binary.

/// 3D-ASCII font - 3D ASCII art style.
pub const THREE_D_ASCII_FLF: &str = include_str!("../../../fonts/3D-ASCII.flf");

/// Alphabet font - simple alphabet letters.
pub const ALPHABET_FLF: &str = include_str!("../../../fonts/Alphabet.flf");

/// ANSI Shadow font - shadow effect style.
pub const ANSI_SHADOW_FLF: &str = include_str!("../../../fonts/ANSI_Shadow.flf");

/// Avatar font - Avatar movie inspired.
pub const AVATAR_FLF: &str = include_str!("../../../fonts/Avatar.flf");

/// Banner font - wide banner style.
pub const BANNER_FLF: &str = include_str!("../../../fonts/Banner.flf");

/// Big font - large block letters.
pub const BIG_FLF: &str = include_str!("../../../fonts/Big.flf");

/// Block font - solid blocks.
pub const BLOCK_FLF: &str = include_str!("../../../fonts/Block.flf");

/// Chunky font - thick chunky letters.
pub const CHUNKY_FLF: &str = include_str!("../../../fonts/Chunky.flf");

/// Colossal font - massive letters.
pub const COLOSSAL_FLF: &str = include_str!("../../../fonts/Colossal.flf");

/// Alligator font - alligator teeth style.
pub const ALLIGATOR_FLF: &str = include_str!("../../../fonts/Alligator.flf");

/// Doom font - Doom game inspired.
pub const DOOM_FLF: &str = include_str!("../../../fonts/Doom.flf");

/// Electronic font - electronic display style.
pub const ELECTRONIC_FLF: &str = include_str!("../../../fonts/Electronic.flf");

/// Epic font - epic dramatic style.
pub const EPIC_FLF: &str = include_str!("../../../fonts/Epic.flf");

/// Graffiti font - street art style.
pub const GRAFFITI_FLF: &str = include_str!("../../../fonts/Graffiti.flf");

/// Ivrit font - Hebrew-inspired style.
pub const IVRIT_FLF: &str = include_str!("../../../fonts/Ivrit.flf");

/// Larry 3D font - 3D perspective style.
pub const LARRY_3D_FLF: &str = include_str!("../../../fonts/Larry_3D.flf");

/// Lean font - thin slanted style.
pub const LEAN_FLF: &str = include_str!("../../../fonts/Lean.flf");

/// Mini font - tiny compact letters.
pub const MINI_FLF: &str = include_str!("../../../fonts/Mini.flf");

/// Ogre font - bold ogre style.
pub const OGRE_FLF: &str = include_str!("../../../fonts/Ogre.flf");

/// Poison font - toxic/poison style.
pub const POISON_FLF: &str = include_str!("../../../fonts/Poison.flf");

/// Roman font - classical Roman style.
pub const ROMAN_FLF: &str = include_str!("../../../fonts/Roman.flf");

/// Script font - cursive handwriting style.
pub const SCRIPT_FLF: &str = include_str!("../../../fonts/Script.flf");

/// Shadow font - letters with shadow.
pub const SHADOW_FLF: &str = include_str!("../../../fonts/Shadow.flf");

/// Slant font - italic-style slant.
pub const SLANT_FLF: &str = include_str!("../../../fonts/Slant.flf");

/// Small font - compact version.
pub const SMALL_FLF: &str = include_str!("../../../fonts/Small.flf");

/// Speed font - fast/motion style.
pub const SPEED_FLF: &str = include_str!("../../../fonts/Speed.flf");

/// Standard FIGlet font - the classic default.
pub const STANDARD_FLF: &str = include_str!("../../../fonts/Standard.flf");

/// Star Wars font - Star Wars movie style.
pub const STAR_WARS_FLF: &str = include_str!("../../../fonts/Star_Wars.flf");

/// Acrobatic font - acrobatic style letters.
pub const ACROBATIC_FLF: &str = include_str!("../../../fonts/Acrobatic.flf");

/// ANSI Regular font - clean ANSI style.
pub const ANSI_REGULAR_FLF: &str = include_str!("../../../fonts/ANSI_Regular.flf");

/// Big Money-ne font - money/currency style.
pub const BIG_MONEY_NE_FLF: &str = include_str!("../../../fonts/Big_Money_ne.flf");

/// BlurVision ASCII font - blurred vision effect.
pub const BLURVISION_ASCII_FLF: &str = include_str!("../../../fonts/BlurVision_ASCII.flf");

/// Doh font - Homer Simpson style.
pub const DOH_FLF: &str = include_str!("../../../fonts/Doh.flf");

/// Bell font - bell-shaped letters.
pub const BELL_FLF: &str = include_str!("../../../fonts/Bell.flf");

/// Puffy font - puffy cloud style.
pub const PUFFY_FLF: &str = include_str!("../../../fonts/Puffy.flf");

/// Rectangles font - rectangular blocks.
pub const RECTANGLES_FLF: &str = include_str!("../../../fonts/Rectangles.flf");

/// Mono 9 font - monospace 9-line (TLF format)
pub const MONO_9_FLF: &str = include_str!("../../../fonts/Mono_9.flf");

/// Mono 12 font - monospace 12-line (TLF format).
pub const MONO_12_FLF: &str = include_str!("../../../fonts/Mono_12.flf");

/// Rebel font - rebellious style (TLF format).
pub const REBEL_FLF: &str = include_str!("../../../fonts/Rebel.flf");

/// Terrace font - terraced/stepped style.
pub const TERRACE_FLF: &str = include_str!("../../../fonts/Terrace.flf");

/// Tmplr font - templated style.
pub const TMPLR_FLF: &str = include_str!("../../../fonts/Tmplr.flf");

/// List of all bundled fonts with their names and content.
pub const BUNDLED_FONTS: &[(&str, &str)] = &[
    ("3D-ASCII", THREE_D_ASCII_FLF),
    ("Alphabet", ALPHABET_FLF),
    ("ANSI Shadow", ANSI_SHADOW_FLF),
    ("Avatar", AVATAR_FLF),
    ("Banner", BANNER_FLF),
    ("Big", BIG_FLF),
    ("Block", BLOCK_FLF),
    ("Chunky", CHUNKY_FLF),
    ("Colossal", COLOSSAL_FLF),
    ("Alligator", ALLIGATOR_FLF),
    ("Doom", DOOM_FLF),
    ("Electronic", ELECTRONIC_FLF),
    ("Epic", EPIC_FLF),
    ("Graffiti", GRAFFITI_FLF),
    ("Ivrit", IVRIT_FLF),
    ("Larry 3D", LARRY_3D_FLF),
    ("Lean", LEAN_FLF),
    ("Mini", MINI_FLF),
    ("Ogre", OGRE_FLF),
    ("Poison", POISON_FLF),
    ("Roman", ROMAN_FLF),
    ("Script", SCRIPT_FLF),
    ("Shadow", SHADOW_FLF),
    ("Slant", SLANT_FLF),
    ("Small", SMALL_FLF),
    ("Speed", SPEED_FLF),
    ("Standard", STANDARD_FLF),
    ("Star Wars", STAR_WARS_FLF),
    ("Acrobatic", ACROBATIC_FLF),
    ("ANSI Regular", ANSI_REGULAR_FLF),
    ("Big Money-ne", BIG_MONEY_NE_FLF),
    ("BlurVision ASCII", BLURVISION_ASCII_FLF),
    ("Doh", DOH_FLF),
    ("Bell", BELL_FLF),
    ("Puffy", PUFFY_FLF),
    ("Rectangles", RECTANGLES_FLF),
    ("Mono 9", MONO_9_FLF),
    ("Mono 12", MONO_12_FLF),
    ("Rebel", REBEL_FLF),
    ("Terrace", TERRACE_FLF),
    ("Tmplr", TMPLR_FLF),
];
