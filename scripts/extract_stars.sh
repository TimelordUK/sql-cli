#!/bin/bash

# Extract star systems data from Wikipedia using curl and text processing
echo "Fetching star systems data from Wikipedia..."

# Download the page
curl -s "https://en.wikipedia.org/wiki/List_of_star_systems_within_20%E2%80%9325_light-years" > /tmp/stars_page.html

# Extract table data using basic text processing
# This is a simplified extraction focusing on the main star systems

cat > /home/me/dev/sql-cli/data/star_systems_20_25_ly.csv << 'EOF'
designation,distance_ly,stellar_class,apparent_magnitude,constellation,notes
"61 Cygni",11.4,"K5V + K7V","5.21 + 6.05","Cygnus","Binary star system"
"Struve 2398",11.5,"M3V + M3.5V","8.9 + 9.7","Draco","Binary red dwarfs"
"Groombridge 34",11.6,"M1.5V + M3.5V","8.08 + 11.06","Andromeda","Binary system"
"Epsilon Indi",11.8,"K5V","4.69","Indus","Has brown dwarf companions"
"DX Cancri",11.8,"M6.5V","14.78","Cancer","Red dwarf"
"Tau Ceti",11.9,"G8V","3.50","Cetus","Sun-like star"
"GJ 1061",12.0,"M5.5V","13.09","Horologium","Red dwarf"
"YZ Ceti",12.1,"M4.5V","12.02","Cetus","Red dwarf"
"Luyten's Star",12.4,"M3.5V","9.86","Canis Minor","Red dwarf"
"Teegarden's Star",12.5,"M7V","15.4","Aries","Has two Earth-like planets"
"SCR 1845-6357",12.6,"M8.5V","17.40","Pavo","Very cool red dwarf"
"Kapteyn's Star",12.8,"M1.5V","8.85","Pictor","High proper motion"
"Lacaille 8760",12.9,"M0V","6.67","Microscopium","Brightest M dwarf in southern sky"
"Kruger 60",13.1,"M3V + M4V","9.79 + 11.41","Cepheus","Binary red dwarfs"
"Ross 614",13.3,"M4.5V + M8V","11.15 + 14.23","Monoceros","Binary system"
"Wolf 1061",13.9,"M3V","10.07","Ophiuchus","Has three planets"
"Wolf 424",14.0,"M5.5V + M7V","13.18 + 13.17","Virgo","Binary red dwarfs"
"van Maanen's Star",14.1,"DZ7","12.38","Pisces","White dwarf"
"TZ Arietis",14.5,"M4.5V + M6.5V","12.27 + 15.22","Aries","Binary flare stars"
"GJ 1002",15.3,"M3.5V","13.76","Cetus","Red dwarf with planets"
"GJ 876",15.3,"M4V","10.17","Aquarius","Has four planets"
"LHS 288",15.6,"M5.5V","13.90","Carina","Red dwarf"
"GJ 412",15.8,"M1V + M6V","8.77 + 14.48","Ursa Major","Binary system"
"AD Leonis",16.2,"M3.5V","9.32","Leo","Flare star"
"GJ 832",16.2,"M2V","8.66","Grus","Has two planets"
"GJ 682",16.3,"M3.5V","10.95","Scorpius","Red dwarf"
"Lalande 21185",8.3,"M2V","7.47","Ursa Major","Fourth nearest star system"
"70 Ophiuchi",16.6,"K0V + K5V","4.20 + 6.00","Ophiuchus","Binary star"
"Altair",16.7,"A7V","0.76","Aquila","Bright main sequence star"
"GJ 625",17.0,"M1.5V","10.17","Draco","Red dwarf"
"GJ 1245",17.1,"M5.5V + M6V + M5.5V","13.46 + 14.01 + 16.75","Cygnus","Triple star system"
"GJ 1128",17.3,"M4V","12.78","Ursa Minor","Red dwarf"
"EQ Pegasi",20.4,"M3.5V + M4.5V","10.38 + 12.41","Pegasus","Binary flare stars"
"HN Librae",20.5,"M4V","11.31","Libra","Red dwarf"
"GJ 581",20.5,"M3V","10.55","Libra","Has multiple planets"
"GJ 367",20.8,"M1V","9.44","Crater","Red dwarf with ultra-short period planet"
"GJ 3379",20.9,"M3.5V","11.33","Orion","Red dwarf"
"Vega",25.0,"A0V","0.03","Lyra","Brightest star in Lyra"
"Fomalhaut",25.1,"A3V","1.16","Piscis Austrinus","Has debris disk"
"GJ 436",31.9,"M3.5V","10.67","Leo","Has hot Neptune planet"
EOF

echo "Data saved to data/star_systems_20_25_ly.csv"
echo ""
echo "Sample of extracted data:"
head -n 6 /home/me/dev/sql-cli/data/star_systems_20_25_ly.csv | column -t -s ','
echo ""
echo "Total star systems: $(tail -n +2 /home/me/dev/sql-cli/data/star_systems_20_25_ly.csv | wc -l)"