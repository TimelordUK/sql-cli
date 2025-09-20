#!/usr/bin/env python3
"""
Sample house prices data to create a smaller representative dataset.
Uses stratified sampling to maintain data distribution.
"""

import csv
import random
from collections import defaultdict
from typing import List, Dict, Any

def read_csv(filename: str) -> tuple[List[str], List[Dict[str, str]]]:
    """Read CSV file and return headers and data."""
    with open(filename, 'r') as f:
        reader = csv.DictReader(f)
        headers = reader.fieldnames
        data = list(reader)
    return headers, data

def stratified_sample(data: List[Dict[str, str]], target_size: int = 5000) -> List[Dict[str, str]]:
    """
    Perform stratified sampling to maintain distribution of key features.
    """
    # Group by ocean_proximity to maintain geographic distribution
    proximity_groups = defaultdict(list)
    for row in data:
        proximity_groups[row['ocean_proximity']].append(row)

    # Also create price bins for stratification
    price_bins = defaultdict(list)
    for row in data:
        try:
            price = float(row['median_house_value'])
            # Create price bins (in 100k increments)
            bin_idx = int(price / 100000)
            price_bins[bin_idx].append(row)
        except (ValueError, KeyError):
            pass

    sampled = []

    # Sample proportionally from each ocean proximity group
    for proximity, rows in proximity_groups.items():
        # Calculate proportion of this group in original data
        proportion = len(rows) / len(data)
        # Sample size for this group
        group_sample_size = int(target_size * proportion)

        if group_sample_size > 0:
            # Further stratify by median_income within each proximity group
            income_groups = defaultdict(list)
            for row in rows:
                try:
                    income = float(row['median_income'])
                    # Create income bins
                    income_bin = int(income)  # Round to nearest integer
                    income_groups[income_bin].append(row)
                except (ValueError, KeyError):
                    income_groups['unknown'].append(row)

            # Sample from each income group proportionally
            group_sampled = []
            for income_bin, income_rows in income_groups.items():
                income_proportion = len(income_rows) / len(rows)
                income_sample_size = max(1, int(group_sample_size * income_proportion))

                if len(income_rows) <= income_sample_size:
                    group_sampled.extend(income_rows)
                else:
                    group_sampled.extend(random.sample(income_rows, income_sample_size))

            sampled.extend(group_sampled[:group_sample_size])

    # Ensure we have exactly the target size
    if len(sampled) > target_size:
        sampled = random.sample(sampled, target_size)
    elif len(sampled) < target_size:
        # Add more samples if needed
        remaining = target_size - len(sampled)
        additional = random.sample(data, min(remaining, len(data)))
        sampled.extend(additional)

    return sampled

def analyze_distribution(data: List[Dict[str, str]], label: str):
    """Print distribution statistics for the dataset."""
    print(f"\n{label} Statistics:")
    print(f"Total rows: {len(data)}")

    # Ocean proximity distribution
    proximity_counts = defaultdict(int)
    for row in data:
        proximity_counts[row['ocean_proximity']] += 1

    print("\nOcean Proximity Distribution:")
    for proximity, count in sorted(proximity_counts.items()):
        percentage = (count / len(data)) * 100
        print(f"  {proximity:15s}: {count:5d} ({percentage:5.1f}%)")

    # Price statistics
    prices = []
    for row in data:
        try:
            prices.append(float(row['median_house_value']))
        except (ValueError, KeyError):
            pass

    if prices:
        print(f"\nPrice Statistics:")
        print(f"  Min:    ${min(prices):,.0f}")
        print(f"  Max:    ${max(prices):,.0f}")
        print(f"  Median: ${sorted(prices)[len(prices)//2]:,.0f}")
        print(f"  Mean:   ${sum(prices)/len(prices):,.0f}")

    # Income statistics
    incomes = []
    for row in data:
        try:
            incomes.append(float(row['median_income']))
        except (ValueError, KeyError):
            pass

    if incomes:
        print(f"\nIncome Statistics:")
        print(f"  Min:    {min(incomes):.2f}")
        print(f"  Max:    {max(incomes):.2f}")
        print(f"  Median: {sorted(incomes)[len(incomes)//2]:.2f}")
        print(f"  Mean:   {sum(incomes)/len(incomes):.2f}")

def write_csv(filename: str, headers: List[str], data: List[Dict[str, str]]):
    """Write data to CSV file."""
    with open(filename, 'w', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=headers)
        writer.writeheader()
        writer.writerows(data)

def main():
    """Main function to sample the house prices data."""
    input_file = 'data/house_prices.csv'
    output_file = 'data/house_prices_sample.csv'
    target_size = 5000  # Target ~100KB file (roughly 5000 rows)

    print(f"Reading {input_file}...")
    headers, data = read_csv(input_file)

    print(f"Original data: {len(data)} rows")

    # Show original distribution
    analyze_distribution(data, "Original Dataset")

    # Perform stratified sampling
    print(f"\nSampling {target_size} rows using stratified sampling...")
    random.seed(42)  # For reproducibility
    sampled_data = stratified_sample(data, target_size)

    # Show sampled distribution
    analyze_distribution(sampled_data, "Sampled Dataset")

    # Write sampled data
    print(f"\nWriting sampled data to {output_file}...")
    write_csv(output_file, headers, sampled_data)

    # Check file size
    import os
    file_size = os.path.getsize(output_file)
    print(f"Output file size: {file_size / 1024:.1f} KB")

    print("\nDone! The sampled data maintains the distribution characteristics of the original.")

if __name__ == "__main__":
    main()