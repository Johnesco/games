"The Great Career Quest" by John Escobedo

The story headline is "An Interactive Resume".
The story genre is "Fantasy".
The story description is "Explore John Escobedo's 25+ year career through a Zork-inspired text adventure. Collect eight career artifacts and return them to the Trophy Case to win."
The release number is 1.

Part 1 - Mechanics

Chapter 1 - Artifacts

An artifact is a kind of thing. An artifact is portable.

Chapter 2 - Darkness

Rule for printing the description of a dark room:
	say "It is pitch black. You are likely to be consumed by imposter syndrome. If only you had some source of light..." instead.

Chapter 3 - Winning

Every turn:
	let N be the number of artifacts in the career trophy case;
	if N is 8:
		say "[line break]As you place the final artifact in the trophy case, the room fills with warm golden light. Every fluorescent bulb in the Career Center seems to burn a little brighter.[paragraph break]A voice echoes through the halls: 'Congratulations, adventurer! You have assembled all eight artifacts of John Escobedo's career -- a journey spanning 25+ years across quality assurance, business analysis, creative design, teaching, and entertainment.[paragraph break]From the QA Dungeons of game testing to the Creative Districts of animation, from healthcare corridors to battle arenas, each artifact tells a piece of the story. Together, they represent a career built on curiosity, adaptability, and the relentless pursuit of quality.[paragraph break]Thank you for playing The Great Career Quest!'[line break]";
		end the story finally.

Chapter 4 - Trophy Case Behavior

After inserting an artifact into the career trophy case:
	let N be the number of artifacts in the career trophy case;
	say "You carefully place [the noun] in the trophy case. [[N] of 8 artifacts collected.]".

Part 2 - The Career Center

Chapter 1 - Entrance

Career Center Entrance is a room. "You are standing west of a gleaming glass building, its facade reflecting the Austin, Texas sun. A brass plaque reads 'JOHN ESCOBEDO CAREER CENTER -- Est. 1993.' The automatic doors to the east stand open, beckoning you inside.[paragraph break]A weathered mailbox sits by the entrance. The faint sound of keyboard clicks and coffee brewing drifts from within."

The brass plaque is scenery in Career Center Entrance. The description is "The plaque reads: 'Quality & Business Analyst | Requirements & Process Specialist | UAT Leadership. 18+ years bridging technical teams, stakeholders, and end-users across healthcare, gaming, e-commerce, and government systems.'"

The weathered mailbox is a container in Career Center Entrance. It is fixed in place and open. The description is "A sturdy mailbox. Inside you notice a folded piece of parchment."

The career summary scroll is in the weathered mailbox. The description is "The scroll reads: 'I specialize in requirements documentation, process analysis, UAT leadership, and cross-functional collaboration. I have built QA and documentation frameworks from scratch, led Agile transitions, and served as a subject matter expert translating complex business needs into testable, actionable requirements. Trilingual communicator (English, Spanish, ASL).'[paragraph break]On the back: 'HINT: Explore the Career Center to find 8 career artifacts. Return them all to the Trophy Case in the lobby to complete The Great Career Quest!'"

Chapter 2 - Lobby

Career Center Lobby is east of Career Center Entrance. "You stand in the spacious lobby of the Career Center. Polished floors reflect the glow of overhead lights. Corridors branch off in every direction:[paragraph break]East leads to the Education Wing.[line break]North leads to the Creative District.[line break]Down leads to the QA Dungeon.[line break]West leads to the Performance Hall.[paragraph break]In the center of the room, a magnificent glass trophy case awaits."

The career trophy case is an open container in Career Center Lobby. It is fixed in place. The description is "A magnificent glass trophy case, about four feet tall, sitting on a marble pedestal. [if the number of artifacts in the career trophy case is 0]It is completely empty, its shelves waiting to display the artifacts of a distinguished career.[otherwise if the number of artifacts in the career trophy case is 8]It gleams magnificently with all eight career artifacts![otherwise]It contains [the number of artifacts in the career trophy case] of 8 artifacts so far.[end if]"

The Lantern of Experience is in Career Center Lobby. "A brass lantern sits on the reception desk, glowing with a warm, steady light." The description is "An old brass lantern that burns with the steady flame of accumulated professional experience. It reads 'Powered by 25+ years of learning on the job.' This should illuminate even the darkest corners of the QA Dungeon."
The Lantern of Experience is lit.

The reception desk is scenery in Career Center Lobby. The description is "A curved reception desk with a guest book. A small sign reads: 'Welcome! Please explore all wings. Remember: collect all 8 artifacts and return them here to the Trophy Case.'"

Part 3 - Education Wing

Chapter 1 - Education Wing Hub

Education Wing is east of Career Center Lobby. "You enter a bright corridor lined with diplomas, certificates, and student work. The air smells of fresh textbooks and dry-erase markers. Classrooms branch off in several directions:[paragraph break]North leads to the San Jacinto Classroom.[line break]East leads to the Art Institute Studio.[line break]South leads to the Vista College Lab.[line break]Up leads to the ASL Academy.[line break]West returns to the lobby."

The diploma wall is scenery in Education Wing. The description is "The wall displays credentials from five institutions: San Jacinto College (1993-95), Art Institute of Houston (1995-96), Vista College Berkeley (2001-02), ASL School (2008-12), and Austin Community College (2015-17). An impressive commitment to lifelong learning."

Chapter 2 - San Jacinto Classroom

San Jacinto Classroom is north of Education Wing. "You enter a computer lab circa 1993, complete with chunky CRT monitors and the gentle hum of cooling fans. A whiteboard displays 'CS 101' and 'Fine Art Fundamentals.' This is San Jacinto College in Pasadena, Texas -- where it all began.[paragraph break]Posters on the wall show early programming concepts alongside figure drawing exercises. An interesting combination."

The CRT monitor is scenery in San Jacinto Classroom. The description is "A thick, beige CRT monitor displaying a C++ compiler. The cursor blinks patiently. San Jacinto College, 1993-1995: Computer Science and Fine Art. The foundation of a career that would bridge the technical and the creative."

Chapter 3 - Art Institute Studio

Art Institute Studio is east of Education Wing. "You step into a design studio filled with drafting tables, typography specimens, and a single SGI workstation running 3D Studio Max. This is the Art Institute of Houston, circa 1995.[paragraph break]Samples of student work line the walls: typography layouts, figure studies, and early 3D renders that look charmingly primitive by today's standards."

The typography specimen is scenery in Art Institute Studio. The description is "A beautifully kerned specimen sheet showcasing various typefaces. Art Institute of Houston, 1995-96: Studies in Fine Art, Typography, Layout, and 3D Max. Where technical precision met artistic vision."

Chapter 4 - Vista College Lab

Vista College Lab is south of Education Wing. "A video editing suite in Berkeley, California. Multiple monitors display timelines filled with clips. A life drawing easel stands in the corner -- because creativity takes many forms.[paragraph break]A teaching assistant's desk near the door is covered in student portfolios and grading rubrics."

The editing timeline is scenery in Vista College Lab. The description is "A complex video editing timeline on a professional monitor. Vista College, Berkeley, 2001-02: Studies in Video Editing and Life Drawing. GPA: 3.143. Also served as a Teaching Assistant, tutoring students in Adobe Photoshop and Illustrator."

Chapter 5 - ASL Academy

ASL Academy is above Education Wing. "You enter a quiet classroom where the walls are covered with ASL handshape charts and Deaf culture posters. Unlike other rooms, this one is intentionally silent -- communication here happens with the hands, face, and body.[paragraph break]A grade report pinned to a corkboard catches your eye. On a shelf near the door, something glimmers."

The Trilingual Amulet is an artifact in ASL Academy. "A shimmering amulet rests on a display shelf, pulsing with three distinct colors." The description is "The Trilingual Amulet glows with three intertwined lights -- one for each language John speaks: English (native), Spanish (conversational), and American Sign Language (conversational). Earned through years of study at ASL School (2008-12) and Austin Community College (2015-17), where he achieved a perfect 4.0 GPA.[paragraph break]Learning ASL was not just academic -- at Communication Services for the Deaf, he immersed himself in Deaf culture to better understand the users he was testing for."

The ASL grade report is scenery in ASL Academy. The description is "The report shows a perfect 4.0 GPA across ASL I through IV at Austin Community College, plus Visual Gestural Communication. Years of dedication to bridging communication gaps."

Part 4 - Creative District

Chapter 1 - Creative District Hub

Creative District is north of Career Center Lobby. "You emerge into a colorful open space that feels part art gallery, part co-working space. Paint-splattered easels share floor space with Wacom tablets and light tables. The creative energy is palpable.[paragraph break]North leads to the Animation Workshop.[line break]East leads to the Design Studio.[line break]West leads to the Teaching Hall.[line break]South returns to the lobby."

Chapter 2 - Animation Workshop

Animation Workshop is north of Creative District. "You enter a professional animation studio. Light tables line the walls, and a Cintiq display shows a half-finished character animation. Cels hang from a drying line overhead.[paragraph break]A sign on the wall reads 'Wild Brain Animation / Diva Productions.' The smell of ink and creativity fills the air."

The Enchanted Animation Cel is an artifact in Animation Workshop. "A single animation cel glows with an otherworldly shimmer on the light table." The description is "The Enchanted Animation Cel shows a perfectly drawn frame of animation, shimmering with creative energy. It represents work at two studios:[paragraph break]WILD BRAIN (San Francisco, 2002) -- Interned at this professional animation studio, gaining hands-on experience with digital coloring workflows and traditional animation using professional equipment.[paragraph break]DIVA PRODUCTIONS (Austin, 2004-05) -- Owned the full animation production pipeline for educational children's content: writing scripts, creating storyboards, building animatics, and delivering final animations."

The light table is scenery in Animation Workshop. The description is "A professional animation light table, still warm to the touch. The tools of traditional animation -- a skill that bridges the gap between art and technology."

Chapter 3 - Design Studio

Design Studio is east of Creative District. "A multi-era design studio that seems to span the late 90s through early 2000s. On one desk, QuarkXPress layouts for a Spanish-language newspaper. On another, early web graphics for an eLearning platform. A Rich Media research station occupies the far wall.[paragraph break]The room tells the story of design's evolution from print to digital."

The newspaper layout is scenery in Design Studio. The description is "A QuarkXPress layout for El Dia Daily Newspaper (Houston, 2004) -- a Spanish-language daily where all work and communication happened in Spanish. Also visible are projects from Mediaplex (San Francisco, 1999-2000, Rich Media research), DigitalThink (2000, eLearning graphics), and Computer Discovery Center (Houston, 2002-04, digital art and photo editing). A diverse portfolio spanning print, web, and multimedia."

Chapter 4 - Teaching Hall

Teaching Hall is west of Creative District. "You enter a warm, well-lit classroom with rows of computers and a large projection screen. Student work is displayed proudly on every wall. This room has seen thousands of students pass through, each one leaving a little more skilled than when they arrived.[paragraph break]A course catalog on the podium lists an impressive range of subjects."

The course catalog is scenery in Teaching Hall. The description is "The catalog lists teaching positions spanning Houston to the Bay Area:[paragraph break]HOUSTON MULTIMEDIA CENTER (1997-98) -- Curriculum development and instruction in Adobe PageMill, HTML, Photoshop, and After Effects.[paragraph break]BAY AREA VIDEO COALITION (1999-2000) -- Expert-level instruction in After Effects and Dreamweaver. Multi-session workshops combining lecture, demo, and hands-on practice.[paragraph break]MEDIA ALLIANCE (San Francisco, 2001-02) -- Workshops on Microsoft Office, Photoshop, and web fundamentals. Managed the computer lab facility.[paragraph break]TELE ATLAS (Menlo Park, 2000-01) -- Developed training programs on data collection methodologies and evaluated trainee performance.[paragraph break]A career of empowering others through knowledge transfer -- a skill that would prove essential in QA leadership."

Part 5 - The QA Dungeon

Chapter 1 - Dungeon Landing

QA Dungeon Landing is below Career Center Lobby. "You descend a spiral staircase into the depths below the Career Center. The fluorescent lights flicker ominously. A sign reads 'THE QA DUNGEON -- The Great Underground Empire of Quality Assurance.'[paragraph break]The air is thick with the tension of unreproduced bugs and ambiguous requirements. Passages lead in several directions:[paragraph break]North leads to the Game Testing Cavern.[line break]East leads to the Virtual Worlds Chamber.[line break]South leads to the Battle Arena.[line break]West leads to the Healthcare Corridor.[line break]Up returns to the lobby.[paragraph break]Some passages ahead look dark. You might need a light source."

Chapter 2 - Game Testing Cavern

Game Testing Cavern is north of QA Dungeon Landing. "You enter a cavern filled with gaming hardware from every era. Stacks of game discs tower precariously. Test matrices cover the walls, annotated in red ink. A desk overflows with 100+ mobile devices of every make and model.[paragraph break]Two company banners hang from the ceiling: 'Super Happy Fun Fun' and 'Aspyr Media.'"

The Golden Bug Report is an artifact in Game Testing Cavern. "A golden document glows atop a stack of game cases." The description is "The Golden Bug Report shimmers with the weight of thousands of meticulously documented defects. It represents the foundation of a QA career:[paragraph break]SUPER HAPPY FUN FUN (Austin, 2006-07) -- Cross-platform compatibility testing across 100+ unique mobile device models. Documented platform-specific bugs in Mantis. Maintained quality oversight across the full product lifecycle.[paragraph break]ASPYR MEDIA (Austin, 2006) -- QA for high-profile Mac game ports including Call of Duty 2 and Civilization IV. Authored detailed test plans, collaborated with dev to isolate Mac-specific issues, and caught localization errors through exhaustive proofreading.[paragraph break]The inscription reads: 'Every bug found in testing is a bug the user never sees.'"

The device stack is scenery in Game Testing Cavern. The description is "Over one hundred mobile devices of various makes and models, each tagged with test results. The sheer variety is staggering -- a testament to the thoroughness required in cross-platform QA."

Chapter 3 - Virtual Worlds Chamber

Virtual Worlds Chamber is a dark room. Virtual Worlds Chamber is east of QA Dungeon Landing. "A vast digital landscape stretches before you -- or rather, a room designed to simulate one. Holographic displays show the virtual world of Second Life: avatars flying, building, and socializing across an infinite digital frontier.[paragraph break]Three promotion certificates are framed on the wall, each one a step up."

The promotion certificates are scenery in Virtual Worlds Chamber. The description is "LINDEN LAB / SECOND LIFE (San Francisco Remote, 2007-2010):[paragraph break]Three promotions in three years -- from frontline First Responder to Community Operations to QA Analyst. Started by managing in-world emergencies and enforcing Terms of Service. Identified operational inefficiencies and built custom diagnostic tools. Played a key role in establishing customer support processes. Promoted to QA Analyst after demonstrating exceptional analytical and diagnostic skills.[paragraph break]A journey from support to QA that proved the value of understanding systems from the user's perspective."

Chapter 4 - Battle Arena

Battle Arena is a dark room. Battle Arena is south of QA Dungeon Landing. "You step into an epic arena that could only be one place: the World of Warcraft support center at Blizzard Entertainment. Holographic displays show Azeroth in all its glory. A Game Master's console glows in the center of the room, ready to resolve the next crisis.[paragraph break]Something glints near the console."

The Game Masters Badge is an artifact in Battle Arena. "A gleaming badge rests beside the Game Master's console." The description is "The Game Master's Badge bears the Blizzard Entertainment crest and the title 'Game Master -- World of Warcraft' (Austin, 2010-2011).[paragraph break]Analyzed and documented cases of fraud and exploitation, providing detailed reports with root cause analysis. Provided compassionate support during emotionally charged escalations -- restoring lost items and accounts while maintaining user trust. De-escalated frustrated users through patient listening. Delivered high-volume support via live chat and ticket systems.[paragraph break]The badge inscription reads: 'For the Horde! (And also the Alliance. We do not discriminate in customer service.)'"

The GM console is scenery in Battle Arena. The description is "A Game Master's console for World of Warcraft. The screen shows a queue of support tickets, each representing a player in need. The tools of empathetic, high-touch customer experience support."

Chapter 5 - Healthcare Corridor

Healthcare Corridor is a dark room. Healthcare Corridor is west of QA Dungeon Landing. "A long, sterile corridor with the unmistakable feel of healthcare technology. HIPAA compliance notices line the walls. Three doors are labeled with company names: DocbookMD, Communication Services for the Deaf, and EverlyWell.[paragraph break]An accessibility testing station occupies one alcove, equipped with screen readers and assistive devices. Something important rests on the testing station."

The Accessibility Scroll is an artifact in Healthcare Corridor. "An ornate scroll rests on the accessibility testing station, radiating an inclusive aura." The description is "The Accessibility Scroll is inscribed with the principles of inclusive design and 508 compliance. It represents expertise earned across three healthcare organizations:[paragraph break]DOCBOOKMD (Austin, 2012-13) -- Founded the QA function for a HIPAA-compliant healthcare messaging app. Analyzed compliance requirements for secure patient health information. Grew the QA department from solo operation to a full team.[paragraph break]COMMUNICATION SERVICES FOR THE DEAF (Austin, 2013-15) -- Developed inaugural QA process for an accessibility-focused video interpreting platform. Invested in learning ASL and Deaf cultural norms to truly understand user needs. Established the company's first quality assurance function.[paragraph break]EVERLYWELL (Austin, 2018) -- Established quality processes for a health tech startup's secure patient portal. Designed unified JIRA workflows and defined initial QA validation criteria.[paragraph break]The scroll whispers: '508 compliance is not a checkbox -- it is a commitment to ensuring everyone can participate.'"

The accessibility station is scenery in Healthcare Corridor. The description is "A testing station equipped with JAWS screen reader, VoiceOver, a refreshable Braille display, and various assistive technologies. Tools for ensuring digital products work for everyone."

Chapter 6 - E-Commerce Lab

E-Commerce Lab is a dark room. E-Commerce Lab is south of Healthcare Corridor. "A bustling lab filled with monitors displaying e-commerce dashboards, coupon codes, and analytics graphs. Everything here is about conversion rates and data integrity. The energy of retail meets the precision of quality assurance.[paragraph break]Company logos for Luna Data Solutions and RetailMeNot adorn the walls."

The analytics dashboard is scenery in E-Commerce Lab. The description is "LUNA DATA SOLUTIONS / RETAILMENOT (Austin, 2015-17) -- Coordinated QA efforts across multiple teams for the high-visibility gift card feature launch. Engineered custom JavaScript tools to validate site-wide analytics and data integrity. Responded to live production issues under pressure, triaging and communicating during critical outages.[paragraph break]Also contracted through Luna Data for HEATWAVE INTERACTIVE (2011-12), where promotion to Test Lead came from documentation quality and team leadership skills."

Chapter 7 - Assessment Chamber

Assessment Chamber is a dark room. Assessment Chamber is east of E-Commerce Lab. "A clean, modern room with assessment terminals lining the walls. Each screen displays aptitude tests and career matching algorithms. This is where YouScience helped students discover their potential.[paragraph break]A stone tablet leans against one of the terminals, inscribed with methodology."

The Agile Manifesto Tablet is an artifact in Assessment Chamber. "A heavy stone tablet leans against a terminal, etched with glowing text." The description is "The Agile Manifesto Tablet is carved from obsidian and etched with the principles of Agile methodology -- but this is not just theory. It represents hands-on Agile leadership:[paragraph break]YOUSCIENCE (Austin, 2017-18) -- As the sole QA resource, built comprehensive quality processes and test infrastructure for the education assessment platform. Created end-to-end test documentation. Introduced standardized user story and acceptance criteria templates that reduced ambiguity.[paragraph break]The tablet's Agile journey continued at GeekSI, where John championed the transition from Waterfall to Agile, coaching distributed teams and gaining leadership buy-in.[paragraph break]Inscribed on the back: 'Individuals and interactions over processes and tools -- but good processes help individuals interact better.'"

Chapter 8 - VA Vault

The VA Vault is a dark room. The VA Vault is north of Assessment Chamber. "You enter a massive, secure vault lined with government documentation. Filing cabinets stretch floor to ceiling, each labeled with VA project codes. A requirements traceability matrix covers an entire wall, mapping every business need to its test coverage.[paragraph break]Six years of meticulous work fill this room. A crystal document gleams on a reading stand."

The Crystal Requirements Document is an artifact in the VA Vault. "A crystalline document floats above a reading stand, refracting light into rainbow patterns." The description is "The Crystal Requirements Document is a masterwork of business analysis, perfectly transparent and multifaceted -- just like good requirements should be.[paragraph break]GEEKSI / VA PROJECTS (Tallahassee FL Remote, 2019-2025) -- 6+ years as Senior QA Test Lead, QA Analyst, and Scrum Master across multiple mission-critical VA healthcare projects.[paragraph break]Became the go-to SME for the Traumatic Brain Injury module. Championed Waterfall-to-Agile transition. Drove standardization of testing practices across distributed teams. Advocated for accessibility as a core quality metric, leading 508 compliance initiatives. Collaborated with automation engineers to validate Cucumber/Selenium scripts. Built requirements traceability documentation mapping every business need to test coverage across web, mainframe, and mobile.[paragraph break]The inscription reads: 'Requirements are the foundation. Everything else is commentary.'"

Chapter 9 - Robotics Lab

Robotics Lab is a dark room. Robotics Lab is east of Game Testing Cavern. "You step into a cutting-edge robotics laboratory. A humanoid robot stands in a testing area, its joints articulated with eerie precision. Monitors display telemetry data, behavior logs, and troubleshooting protocols.[paragraph break]This is the frontier -- where QA meets the physical world."

The robot telemetry is scenery in Robotics Lab. The description is "INSIGHT GLOBAL / APPTRONIK (Austin, 2025) -- Robot Tele-Operator for humanoid robot development. Translated observed robot behaviors into clear, actionable bug reports. Proactively authored troubleshooting protocols and SOPs where none existed. Bridged operations and engineering communication. Reduced physical testing environment setup time by 75 percent through standardized preparation patterns.[paragraph break]Reached full productivity in a novel role with minimal onboarding -- because 18+ years of QA teaches you how to learn fast."

Part 6 - Performance Hall

Chapter 1 - Performance Hall Hub

Performance Hall is west of Career Center Lobby. "You step into a vibrant entertainment venue. Stage lights cast colorful patterns across the floor. The sound system hums with potential. Three stages are visible from here, each representing a different facet of performance and event coordination.[paragraph break]North leads to the DJ Booth.[line break]South leads to the Karaoke Stage.[line break]West leads to the Event Space.[line break]East returns to the lobby."

Chapter 2 - DJ Booth

DJ Booth is north of Performance Hall. "A professional DJ setup dominates this room: turntables, CDJs, a mixer, and speakers that could shake the building. Setlists for Danceversity, Dance International, and Synergy Dance Studio are pinned to a corkboard. The energy of a hundred dance nights lingers in the air.[paragraph break]A magnificent turntable sits on a golden stand."

The Turntable of Coordination is an artifact in DJ Booth. "A legendary turntable sits on a golden stand, softly spinning." The description is "The Turntable of Coordination spins with perfect rhythm, representing the art of reading a room and adapting in real-time.[paragraph break]FREELANCE DJ (Austin, 2011-12) -- Created DJ setlists for Danceversity, Dance International, and Synergy Dance Studio. Adapted to various in-house sound systems and venue constraints. Coordinated with event organizers to align music selection with event tone and timing.[paragraph break]The skills of a DJ -- reading the room, adapting on the fly, managing technical complexity while keeping the audience engaged -- are the same skills that make a great QA lead.[paragraph break]The plaque reads: 'Every great event is a live test of coordination, timing, and user experience.'"

Chapter 3 - Karaoke Stage

Karaoke Stage is south of Performance Hall. "A spotlight illuminates a karaoke stage, complete with a massive song catalog, professional monitors, and a mic that is still warm. The audience area is set up for maximum participation -- because karaoke is about bringing everyone into the experience.[paragraph break]This stage is still active. The setlist for tonight is being prepared."

The karaoke mic is scenery in Karaoke Stage. The description is "KARAOKE HOST (Austin, 2025-Present) -- The current gig. Coordinating event flow and timing while adapting to audience needs in real-time. Managing technical setup including sound systems, software, and troubleshooting. Creating an inclusive, welcoming atmosphere that encourages participation from everyone.[paragraph break]It is not so different from facilitating a sprint review, when you think about it."

Chapter 4 - Event Space

Event Space is west of Performance Hall. "A multi-purpose event venue that once hosted weekly dance events. The layout is optimized for flow: a stage area, a dance floor, and a social zone. Photos on the wall show packed houses and happy dancers.[paragraph break]The Kick Butt Coffee logo is prominently displayed."

The event photos are scenery in Event Space. The description is "KICK BUTT COFFEE (Austin, 2011-12) -- Event Coordinator and Host. Organized and ran weekly events, coordinating DJs, venue logistics, and audience engagement. Set up technical infrastructure for seamless integration with house audio systems. Managed event promotion and community communication through social media.[paragraph break]Event coordination is project management in party form: stakeholders, timelines, logistics, and the ever-present risk of things going sideways."

Part 7 - Special Rules

Chapter 1 - Help Command

Asking for help is an action applying to nothing.
Understand "help" or "hint" or "hints" as asking for help.

Carry out asking for help:
	say "WELCOME TO THE GREAT CAREER QUEST[paragraph break]You are exploring the career of John Escobedo through this interactive text adventure.[paragraph break]COMMANDS:[line break] Move: go north/south/east/west/up/down (or n/s/e/w/u/d)[line break] Look: look (or l)[line break] Examine: examine [bracket]thing[close bracket] (or x [bracket]thing[close bracket])[line break] Take: take [bracket]thing[close bracket][line break] Inventory: inventory (or i)[line break] Put in case: put [bracket]thing[close bracket] in trophy case[paragraph break]GOAL: Find all 8 career artifacts and place them in the Trophy Case in the Career Center Lobby.[paragraph break]TIP: Some rooms in the QA Dungeon are dark. You will need the Lantern of Experience from the lobby to explore them safely.[paragraph break]ARTIFACTS: [the number of artifacts carried by the player + the number of artifacts in the career trophy case] of 8 found so far."

Chapter 2 - Skills Command

Checking skills is an action applying to nothing.
Understand "skills" or "resume" or "qualifications" as checking skills.

Carry out checking skills:
	say "JOHN ESCOBEDO -- KEY SKILLS[paragraph break]Business Analysis: Requirements Elicitation, User Stories, Process Mapping, Gap Analysis, Stakeholder Management, Change Management[paragraph break]QA Leadership: Test Planning, Regression Testing, UAT, CI/CD Testing, Accessibility/508 Testing, Agile/Scrum[paragraph break]Tools: JIRA/Xray, TestRail, GitHub, Chrome DevTools, SQL/JQL, JavaScript/HTML/CSS, Postman[paragraph break]Communication: Trilingual (English, Spanish, ASL), Cross-functional Collaboration, Mentoring, Public Speaking[paragraph break]Industries: Healthcare, Gaming, E-Commerce, Education, Government/VA, Robotics[paragraph break]Type 'examine' on items you find to learn more about specific accomplishments."

Chapter 3 - Contact Command

Showing contact is an action applying to nothing.
Understand "contact" or "email" or "phone" or "linkedin" as showing contact.

Carry out showing contact:
	say "CONTACT JOHN ESCOBEDO[paragraph break]Email: letmeshowyou@gmail.com[line break]Phone: (512) 299-3269[line break]LinkedIn: linkedin.com/in/johnesco[line break]Location: Manor, TX"

Chapter 4 - Scenery Defaults

Rule for deciding whether all includes scenery: it does not.
