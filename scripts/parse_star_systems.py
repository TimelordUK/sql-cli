#!/usr/bin/env python3
"""
Parse Wikipedia star systems data (20-25 light-years) into CSV format.
"""

import csv
import re
import requests
from bs4 import BeautifulSoup
from typing import List, Dict, Any

def clean_text(text: str) -> str:
    """Remove references like [1], extra whitespace, and clean up text."""
    # Remove reference brackets [1], [2], etc.
    text = re.sub(r'\[\d+\]', '', text)
    # Remove non-breaking spaces
    text = text.replace('\xa0', ' ')
    # Clean up whitespace
    text = ' '.join(text.split())
    return text.strip()

def parse_star_systems() -> List[Dict[str, Any]]:
    """Parse the Wikipedia page for star systems within 20-25 light-years."""
    url = "https://en.wikipedia.org/wiki/List_of_star_systems_within_20%E2%80%9325_light-years"

    response = requests.get(url)
    soup = BeautifulSoup(response.content, 'html.parser')

    # Find the main table with star systems
    tables = soup.find_all('table', {'class': 'wikitable'})

    star_systems = []

    for table in tables:
        # Skip tables that don't look like star data
        headers = table.find_all('th')
        if not headers:
            continue

        # Check if this is a star systems table by looking for expected columns
        header_text = [clean_text(h.get_text()) for h in headers]

        # Process rows
        rows = table.find_all('tr')[1:]  # Skip header row

        for row in rows:
            cells = row.find_all(['td', 'th'])
            if len(cells) < 3:  # Skip rows with too few cells
                continue

            # Extract data from cells
            row_data = {}

            # Try to extract common fields
            cell_texts = [clean_text(cell.get_text()) for cell in cells]

            # Typical structure varies, but generally:
            # Designation, Distance (ly), Stellar class, Apparent magnitude, etc.

            if len(cell_texts) >= 4:
                # Basic extraction
                row_data['designation'] = cell_texts[0] if cell_texts[0] else ''

                # Extract distance (look for numeric values with 'ly' or just numbers)
                for i, text in enumerate(cell_texts[1:4], 1):
                    if any(char.isdigit() for char in text):
                        # Try to extract distance
                        dist_match = re.search(r'([\d.]+)', text)
                        if dist_match:
                            row_data['distance_ly'] = dist_match.group(1)
                            break

                # Extract other fields
                row_data['stellar_class'] = cell_texts[2] if len(cell_texts) > 2 else ''
                row_data['apparent_magnitude'] = cell_texts[3] if len(cell_texts) > 3 else ''

                # Additional fields if available
                if len(cell_texts) > 4:
                    row_data['absolute_magnitude'] = cell_texts[4]
                if len(cell_texts) > 5:
                    row_data['mass'] = cell_texts[5]
                if len(cell_texts) > 6:
                    row_data['notes'] = cell_texts[6]

                # Only add if we have a designation
                if row_data.get('designation'):
                    star_systems.append(row_data)

    return star_systems

def save_to_csv(star_systems: List[Dict[str, Any]], filename: str):
    """Save the star systems data to a CSV file."""
    if not star_systems:
        print("No star systems data found")
        return

    # Get all unique keys
    all_keys = set()
    for system in star_systems:
        all_keys.update(system.keys())

    # Sort keys for consistent column order
    fieldnames = sorted(all_keys)

    with open(filename, 'w', newline='', encoding='utf-8') as csvfile:
        writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(star_systems)

    print(f"Saved {len(star_systems)} star systems to {filename}")

def main():
    """Main function to parse and save star systems data."""
    print("Fetching star systems data from Wikipedia...")
    star_systems = parse_star_systems()

    if star_systems:
        print(f"Found {len(star_systems)} star systems")

        # Save to CSV
        output_file = "star_systems_20_25_ly.csv"
        save_to_csv(star_systems, output_file)

        # Print sample
        print("\nFirst 5 entries:")
        for i, system in enumerate(star_systems[:5], 1):
            print(f"{i}. {system.get('designation', 'Unknown')}: "
                  f"{system.get('distance_ly', 'N/A')} light-years")
    else:
        print("No star systems data could be extracted")

if __name__ == "__main__":
    main()