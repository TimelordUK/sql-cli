#!/usr/bin/env python3
"""
Production-Quality VWAP Simulator
Includes: Participation monitoring, urgency-based aggression, fade/retry, slippage tracking
This is what real algo engines do
"""

import json
import csv
import random
from datetime import datetime, timedelta
from typing import List, Dict, Any, Tuple
from enum import Enum

class Urgency(Enum):
    """Algo urgency levels based on participation"""
    PASSIVE = "PASSIVE"      # Ahead of schedule
    NORMAL = "NORMAL"        # On track
    URGENT = "URGENT"        # Behind schedule
    CRITICAL = "CRITICAL"    # Way behind - must catch up

class OrderInstruction(Enum):
    """SOR instructions based on urgency"""
    POST_ONLY = "POST_ONLY"           # Passive - provide liquidity
    LIMIT_IOC = "LIMIT_IOC"           # Normal - take at limit
    MARKET_IOC = "MARKET_IOC"         # Urgent - take liquidity
    SWEEP = "SWEEP"                   # Critical - take all venues

def calculate_vwap_schedule(total_quantity: int, hours: int = 7) -> List[int]:
    """Calculate expected VWAP participation schedule"""
    # Realistic intraday volume curve (U-shaped)
    participation = [
        0.15,  # 9:00-10:00 - High open
        0.10,  # 10:00-11:00
        0.08,  # 11:00-12:00
        0.07,  # 12:00-13:00 - Lunch dip
        0.08,  # 13:00-14:00
        0.10,  # 14:00-15:00
        0.12,  # 15:00-16:00
        0.30,  # 16:00-17:00 - Close & auction
    ]
    
    schedule = []
    for pct in participation[:hours]:
        schedule.append(int(total_quantity * pct))
    
    # Adjust last hour for rounding
    schedule[-1] = total_quantity - sum(schedule[:-1])
    
    return schedule

class ProductionVWAPSimulator:
    """Production-quality VWAP algo simulator"""
    
    def __init__(self, client_order_id: str, quantity: int, ticker: str = "ASML.AS"):
        self.client_order_id = client_order_id
        self.quantity = quantity
        self.ticker = ticker
        self.schedule = calculate_vwap_schedule(quantity)
        self.snapshots = []
        self.record_id = 0
        
        # Tracking
        self.total_filled = 0
        self.total_value = 0.0
        self.expected_filled = 0
        self.slippage_bps = 0.0
        
        # Market state
        self.market_price = 650.00
        self.market_volume = 0
        
    def add_snapshot(self, order: Dict, event_type: str, timestamp: datetime, 
                    urgency: Urgency = None, metadata: Dict = None):
        """Add snapshot with metadata"""
        snapshot = order.copy()
        snapshot['snapshot_time'] = timestamp.isoformat()
        snapshot['event_type'] = event_type
        snapshot['record_id'] = f"REC_{self.record_id:08d}"
        snapshot['market_price'] = self.market_price
        
        if urgency:
            snapshot['urgency'] = urgency.value
            
        if metadata:
            snapshot.update(metadata)
            
        self.snapshots.append(snapshot)
        self.record_id += 1
        
    def calculate_urgency(self, hour: int, filled: int) -> Tuple[Urgency, float]:
        """Calculate urgency based on participation rate"""
        # Expected fill by this hour
        expected = sum(self.schedule[:hour+1])
        
        # Participation rate (% behind/ahead)
        if expected > 0:
            participation_rate = (filled / expected) * 100
        else:
            participation_rate = 100
        
        # Determine urgency
        if participation_rate >= 95:
            urgency = Urgency.PASSIVE
        elif participation_rate >= 85:
            urgency = Urgency.NORMAL
        elif participation_rate >= 70:
            urgency = Urgency.URGENT
        else:
            urgency = Urgency.CRITICAL
            
        shortfall = expected - filled
        
        return urgency, shortfall
    
    def generate_execution(self) -> List[Dict]:
        """Generate complete VWAP execution with participation monitoring"""
        
        # Start time
        current_time = datetime.now().replace(hour=9, minute=0, second=0, microsecond=0)
        
        print("PRODUCTION VWAP EXECUTION")
        print("=" * 80)
        print(f"Order: {self.quantity:,} shares of {self.ticker}")
        print(f"Schedule: {self.schedule}")
        print()
        
        # 1. CLIENT ORDER
        client_order = {
            'order_id': f'CLIENT_{self.client_order_id}',
            'parent_order_id': None,
            'client_order_id': self.client_order_id,
            'order_level': 0,
            'order_type': 'CLIENT',
            'ticker': self.ticker,
            'side': 'Buy',
            'quantity': self.quantity,
            'filled_quantity': 0,
            'remaining_quantity': self.quantity,
            'average_price': 0.0,
            'state': 'PENDING',
            'client_name': 'Wellington Management'
        }
        
        self.add_snapshot(client_order, 'NEW', current_time)
        
        # 2. ALGO ACCEPTS
        current_time += timedelta(seconds=1)
        client_order['state'] = 'ACCEPTED'
        self.add_snapshot(client_order, 'ACCEPTED', current_time)
        
        # 3. ALGO PARENT
        algo_parent = {
            'order_id': 'ALGO_100001',
            'parent_order_id': client_order['order_id'],
            'client_order_id': self.client_order_id,
            'order_level': 1,
            'order_type': 'ALGO_PARENT',
            'ticker': self.ticker,
            'side': 'Buy',
            'quantity': self.quantity,
            'filled_quantity': 0,
            'remaining_quantity': self.quantity,
            'average_price': 0.0,
            'state': 'WORKING',
            'algo_strategy': 'VWAP',
            'participation_target': 'InLine'
        }
        self.add_snapshot(algo_parent, 'NEW', current_time)
        
        # 4. EXECUTE HOURLY SLICES
        slice_counter = 200001
        sor_counter = 300001
        
        for hour in range(len(self.schedule)):
            if self.total_filled >= self.quantity:
                break
                
            current_time = datetime.now().replace(hour=9+hour, minute=0, second=0, microsecond=0)
            hour_target = self.schedule[hour]
            
            # Calculate urgency
            urgency, shortfall = self.calculate_urgency(hour, self.total_filled)
            
            print(f"\nHOUR {hour+1} (1{hour+9}:00)")
            print(f"  Target: {hour_target:,} | Total Target: {sum(self.schedule[:hour+1]):,}")
            print(f"  Filled: {self.total_filled:,} | Shortfall: {shortfall:,}")
            print(f"  Urgency: {urgency.value}")
            
            # Determine slice sizes based on urgency
            if urgency == Urgency.CRITICAL:
                # Send large aggressive slices
                slice_size = min(shortfall, self.quantity - self.total_filled)
                instruction = OrderInstruction.SWEEP
                num_slices = 1
            elif urgency == Urgency.URGENT:
                # Send medium slices more frequently
                slice_size = min(hour_target // 2, self.quantity - self.total_filled)
                instruction = OrderInstruction.MARKET_IOC
                num_slices = 3
            elif urgency == Urgency.NORMAL:
                # Normal slicing
                slice_size = hour_target // 4
                instruction = OrderInstruction.LIMIT_IOC
                num_slices = 4
            else:  # PASSIVE
                # Small slices, post liquidity
                slice_size = hour_target // 6
                instruction = OrderInstruction.POST_ONLY
                num_slices = 6
            
            hour_filled = 0
            
            for slice_num in range(num_slices):
                if self.total_filled >= self.quantity or hour_filled >= hour_target:
                    break
                    
                # Timing within hour
                current_time = datetime.now().replace(
                    hour=9+hour, 
                    minute=(60//num_slices) * slice_num,
                    second=0
                )
                
                # Market moves
                self.market_price += random.uniform(-0.2, 0.2)
                
                # Create slice
                actual_slice_size = min(slice_size, self.quantity - self.total_filled)
                
                slice_order = {
                    'order_id': f'SLICE_{slice_counter}',
                    'parent_order_id': 'ALGO_100001',
                    'client_order_id': self.client_order_id,
                    'order_level': 2,
                    'order_type': 'ALGO_SLICE',
                    'ticker': self.ticker,
                    'side': 'Buy',
                    'quantity': actual_slice_size,
                    'filled_quantity': 0,
                    'remaining_quantity': actual_slice_size,
                    'average_price': 0.0,
                    'state': 'PENDING',
                    'urgency': urgency.value,
                    'instruction': instruction.value
                }
                
                self.add_snapshot(slice_order, 'NEW', current_time, urgency, 
                                {'participation_shortfall': shortfall})
                slice_counter += 1
                
                # Route to SOR
                current_time += timedelta(milliseconds=10)
                slice_filled, slice_value = self.execute_sor(
                    slice_order, instruction, current_time, sor_counter
                )
                sor_counter += 10
                
                # Update slice
                current_time += timedelta(milliseconds=50)
                slice_order['filled_quantity'] = slice_filled
                slice_order['remaining_quantity'] = actual_slice_size - slice_filled
                slice_order['average_price'] = slice_value / slice_filled if slice_filled > 0 else 0
                slice_order['state'] = 'FILLED' if slice_filled >= actual_slice_size else 'PARTIAL'
                
                self.add_snapshot(slice_order, 'SLICE_COMPLETE', current_time, urgency)
                
                hour_filled += slice_filled
                self.total_filled += slice_filled
                self.total_value += slice_value
                
                # Update parent
                self.update_parent_orders(algo_parent, client_order, current_time, urgency)
                
                # Check if we need to get more aggressive
                if hour_filled < hour_target * 0.5 and slice_num > num_slices // 2:
                    print(f"    ⚠️ Behind target - increasing urgency")
                    urgency = Urgency(min(urgency.value, Urgency.CRITICAL.value))
            
            print(f"  Hour Result: Filled {hour_filled:,}/{hour_target:,}")
        
        # Final status
        print(f"\n{'='*80}")
        print(f"FINAL RESULT:")
        print(f"  Filled: {self.total_filled:,}/{self.quantity:,} ({self.total_filled/self.quantity*100:.1f}%)")
        print(f"  VWAP: {self.total_value/self.total_filled if self.total_filled > 0 else 0:.2f}")
        print(f"  Slippage: {self.slippage_bps:.1f} bps")
        
        return self.snapshots
    
    def execute_sor(self, slice_order: Dict, instruction: OrderInstruction, 
                    timestamp: datetime, sor_counter: int) -> Tuple[int, float]:
        """Execute slice through SOR with realistic outcomes"""
        
        filled = 0
        value = 0.0
        
        # Venue selection based on instruction urgency
        if instruction == OrderInstruction.SWEEP:
            venues = ['NYSE', 'NASDAQ', 'ARCA', 'BATS', 'DARK']  # Hit all
        elif instruction == OrderInstruction.MARKET_IOC:
            venues = ['NYSE', 'NASDAQ', 'ARCA']  # Major venues
        elif instruction == OrderInstruction.LIMIT_IOC:
            venues = ['DARK', 'NYSE']  # Dark first
        else:  # POST_ONLY
            venues = ['ARCA']  # Single venue passive
        
        target_qty = slice_order['quantity']
        
        for venue in venues:
            if filled >= target_qty:
                break
                
            venue_qty = target_qty // len(venues)
            
            # Create SOR order
            sor_order = {
                'order_id': f'SOR_{sor_counter}',
                'parent_order_id': slice_order['order_id'],
                'client_order_id': self.client_order_id,
                'order_level': 3,
                'order_type': 'ROUTE',
                'ticker': self.ticker,
                'side': 'Buy',
                'quantity': venue_qty,
                'filled_quantity': 0,
                'remaining_quantity': venue_qty,
                'average_price': 0.0,
                'state': 'PENDING',
                'venue': venue,
                'instruction': instruction.value
            }
            
            self.add_snapshot(sor_order, 'NEW', timestamp)
            sor_counter += 1
            
            # Simulate execution based on urgency
            if instruction == OrderInstruction.SWEEP:
                # Always fills but with slippage
                fill_price = self.market_price + random.uniform(0.02, 0.05)
                sor_order['filled_quantity'] = venue_qty
                sor_order['average_price'] = fill_price
                sor_order['state'] = 'FILLED'
                filled += venue_qty
                value += venue_qty * fill_price
                self.slippage_bps += 5  # Track slippage
                
            elif instruction == OrderInstruction.MARKET_IOC:
                # Usually fills, some slippage
                if random.random() < 0.85:
                    fill_price = self.market_price + random.uniform(0.01, 0.03)
                    sor_order['filled_quantity'] = venue_qty
                    sor_order['average_price'] = fill_price
                    sor_order['state'] = 'FILLED'
                    filled += venue_qty
                    value += venue_qty * fill_price
                    self.slippage_bps += 2
                else:
                    # FADE
                    sor_order['state'] = 'FADE'
                    self.add_snapshot(sor_order, 'VENUE_FADE', timestamp + timedelta(milliseconds=20),
                                    metadata={'fade_reason': 'Liquidity exhausted'})
                    
            elif instruction == OrderInstruction.LIMIT_IOC:
                # Sometimes fills, minimal slippage
                if random.random() < 0.7:
                    fill_price = self.market_price + random.uniform(-0.01, 0.01)
                    
                    # Might be partial
                    if random.random() < 0.3:
                        partial_qty = venue_qty // 2
                        sor_order['filled_quantity'] = partial_qty
                        sor_order['average_price'] = fill_price
                        sor_order['state'] = 'PARTIAL'
                        filled += partial_qty
                        value += partial_qty * fill_price
                    else:
                        sor_order['filled_quantity'] = venue_qty
                        sor_order['average_price'] = fill_price
                        sor_order['state'] = 'FILLED'
                        filled += venue_qty
                        value += venue_qty * fill_price
                        
            else:  # POST_ONLY
                # Rarely fills immediately
                if random.random() < 0.3:
                    fill_price = self.market_price - 0.01  # Better price
                    sor_order['filled_quantity'] = venue_qty
                    sor_order['average_price'] = fill_price
                    sor_order['state'] = 'FILLED'
                    filled += venue_qty
                    value += venue_qty * fill_price
                    self.slippage_bps -= 1  # Negative slippage (good)
            
            timestamp += timedelta(milliseconds=30)
            self.add_snapshot(sor_order, f'VENUE_{sor_order["state"]}', timestamp)
        
        return filled, value
    
    def update_parent_orders(self, algo_parent: Dict, client_order: Dict, 
                           timestamp: datetime, urgency: Urgency):
        """Update parent and client orders"""
        
        # Update algo parent
        algo_parent['filled_quantity'] = self.total_filled
        algo_parent['remaining_quantity'] = self.quantity - self.total_filled
        algo_parent['average_price'] = self.total_value / self.total_filled if self.total_filled > 0 else 0
        algo_parent['state'] = 'FILLED' if self.total_filled >= self.quantity else 'WORKING'
        
        # Track participation
        if urgency in [Urgency.URGENT, Urgency.CRITICAL]:
            algo_parent['participation_target'] = 'BEHIND'
        elif urgency == Urgency.PASSIVE:
            algo_parent['participation_target'] = 'AHEAD'
        else:
            algo_parent['participation_target'] = 'INLINE'
            
        self.add_snapshot(algo_parent, 'ALGO_UPDATE', timestamp, urgency)
        
        # Update client order
        client_order['filled_quantity'] = self.total_filled
        client_order['remaining_quantity'] = self.quantity - self.total_filled
        client_order['average_price'] = algo_parent['average_price']
        client_order['state'] = algo_parent['state']
        
        self.add_snapshot(client_order, 'CLIENT_UPDATE', timestamp)

def main():
    """Generate production VWAP execution"""
    
    sim = ProductionVWAPSimulator(
        client_order_id='PROD_20241216_001',
        quantity=100000,
        ticker='ASML.AS'
    )
    
    snapshots = sim.generate_execution()
    
    # Export
    with open('production_vwap.json', 'w') as f:
        json.dump(snapshots, f, indent=2, default=str)
    
    # Export CSV
    if snapshots:
        keys = set()
        for s in snapshots:
            keys.update(s.keys())
        
        with open('production_vwap.csv', 'w', newline='') as f:
            writer = csv.DictWriter(f, fieldnames=sorted(keys))
            writer.writeheader()
            writer.writerows(snapshots)
    
    print(f"\n✅ Generated {len(snapshots)} snapshots")
    print("📁 Files: production_vwap.csv, production_vwap.json")
    
    # Analysis
    urgency_events = {}
    for s in snapshots:
        if 'urgency' in s:
            urg = s['urgency']
            urgency_events[urg] = urgency_events.get(urg, 0) + 1
    
    print(f"\n📊 Urgency Distribution:")
    for urg, count in urgency_events.items():
        print(f"  {urg}: {count}")

if __name__ == "__main__":
    main()