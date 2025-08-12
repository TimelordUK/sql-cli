#!/usr/bin/env python3
"""
Enhanced SOR Simulator with Realistic Market Behavior
Models: Fade, Partial Fills, Retry Logic, Market Impact
"""

import json
import csv
import random
from datetime import datetime, timedelta
from typing import List, Dict, Any, Optional, Tuple
from dataclasses import dataclass
from enum import Enum
import copy

# ============== ORDER TYPES ==============
class OrderInstruction(Enum):
    """Order execution instructions"""
    EOE = "ExecuteOrEliminate"  # Immediate or cancel
    FOK = "FillOrKill"          # All or nothing
    IOC = "ImmediateOrCancel"   # Partial fills OK
    DAY = "Day"                 # Good for day
    LIMIT = "Limit"             # Passive limit order

class VenueResponse(Enum):
    """Venue execution responses"""
    FILLED = "FILLED"
    PARTIAL = "PARTIAL"
    FADE = "FADE"               # Someone else got liquidity
    REJECTED = "REJECTED"       # Price away from market
    NO_LIQUIDITY = "NO_LIQUIDITY"

# ============== MARKET SIMULATOR ==============
class EnhancedMarketSimulator:
    """Simulates realistic market with liquidity and competition"""
    
    def __init__(self, base_price: float = 650.0):
        self.base_price = base_price
        self.current_price = base_price
        self.bid = base_price - 0.01
        self.ask = base_price + 0.01
        self.available_liquidity = {}  # Per venue
        self.competition_level = 0.5   # 0-1, higher = more competition
        
    def tick(self) -> Tuple[float, float, float, Dict[str, int]]:
        """Generate market tick with liquidity"""
        # Price movement
        move = random.gauss(0, 0.1) - (self.current_price - self.base_price) * 0.01
        self.current_price += move
        
        # Spread
        spread = random.uniform(0.01, 0.05)
        self.bid = round(self.current_price - spread/2, 2)
        self.ask = round(self.current_price + spread/2, 2)
        
        # Available liquidity at top of book per venue
        liquidity = {
            'NYSE': random.randint(500, 5000),
            'NASDAQ': random.randint(500, 5000),
            'ARCA': random.randint(200, 2000),
            'BATS': random.randint(300, 3000),
            'DARK': random.randint(1000, 10000),  # Dark pools often have more
            'IEX': random.randint(100, 1000)
        }
        
        self.available_liquidity = liquidity
        return self.current_price, self.bid, self.ask, liquidity

# ============== ENHANCED VENUE ==============
class EnhancedVenue:
    """Venue with realistic execution behavior"""
    
    def __init__(self, name: str, fade_probability: float = 0.1):
        self.name = name
        self.fade_probability = fade_probability
        self.partial_fill_probability = 0.2
        self.available_liquidity = 0
        
    def execute_order(self, order: Dict, market: EnhancedMarketSimulator, 
                     instruction: OrderInstruction) -> Tuple[VenueResponse, int, float]:
        """
        Execute order with realistic outcomes
        Returns: (response_type, filled_quantity, fill_price)
        """
        _, bid, ask, liquidity = market.tick()
        self.available_liquidity = liquidity.get(self.name, 1000)
        
        # Check for fade (someone else took liquidity)
        if random.random() < self.fade_probability * market.competition_level:
            return VenueResponse.FADE, 0, 0.0
        
        # Check available liquidity
        if self.available_liquidity == 0:
            return VenueResponse.NO_LIQUIDITY, 0, 0.0
        
        # Determine fill quantity
        requested_qty = order['quantity']
        
        if instruction == OrderInstruction.EOE:
            # Execute or Eliminate - take what's available immediately
            if self.available_liquidity < requested_qty:
                # Partial fill or nothing based on venue rules
                if random.random() < 0.3:  # 30% chance venue rejects partial on EOE
                    return VenueResponse.REJECTED, 0, 0.0
                else:
                    fill_qty = self.available_liquidity
                    fill_price = ask if order['side'] == 'Buy' else bid
                    return VenueResponse.PARTIAL, fill_qty, fill_price
            else:
                # Full fill
                fill_price = ask if order['side'] == 'Buy' else bid
                return VenueResponse.FILLED, requested_qty, fill_price
                
        elif instruction == OrderInstruction.IOC:
            # Immediate or Cancel - partial fills OK
            fill_qty = min(requested_qty, self.available_liquidity)
            if fill_qty > 0:
                fill_price = ask if order['side'] == 'Buy' else bid
                response = VenueResponse.FILLED if fill_qty == requested_qty else VenueResponse.PARTIAL
                return response, fill_qty, fill_price
            else:
                return VenueResponse.NO_LIQUIDITY, 0, 0.0
        
        else:  # Default behavior
            fill_qty = min(requested_qty, self.available_liquidity)
            fill_price = ask if order['side'] == 'Buy' else bid
            return VenueResponse.FILLED, fill_qty, fill_price

# ============== ENHANCED SOR ==============
class EnhancedSmartOrderRouter:
    """SOR with retry logic and smart routing"""
    
    def __init__(self, tick_db, venues: Dict[str, EnhancedVenue], market: EnhancedMarketSimulator):
        self.tick_db = tick_db
        self.venues = venues
        self.market = market
        self.order_counter = 5000
        self.max_retries = 2
        
    def route_order(self, algo_child: Dict, timestamp: datetime) -> Tuple[List[Dict], int, datetime]:
        """
        Route order with smart venue selection and retry logic
        Returns: (execution_records, total_filled, final_timestamp)
        """
        execution_records = []
        total_filled = 0
        remaining_qty = algo_child['quantity']
        retry_count = 0
        
        print(f"\n🔀 SOR: Received {algo_child['order_id']} for {algo_child['quantity']:,} shares")
        
        while remaining_qty > 0 and retry_count <= self.max_retries:
            if retry_count > 0:
                print(f"  🔄 SOR: Retry {retry_count} - {remaining_qty:,} shares remaining")
                timestamp += timedelta(milliseconds=500)  # Wait before retry
            
            # Get current market state
            price, bid, ask, liquidity = self.market.tick()
            
            # Smart venue selection based on liquidity
            venue_ranking = sorted(liquidity.items(), key=lambda x: x[1], reverse=True)
            selected_venues = [v[0] for v in venue_ranking[:3]]  # Top 3 by liquidity
            
            # Determine order instruction based on urgency
            if retry_count == 0:
                instruction = OrderInstruction.EOE  # Aggressive first attempt
            else:
                instruction = OrderInstruction.IOC  # More flexible on retries
            
            print(f"  📊 Market: {price:.2f} ({bid:.2f}/{ask:.2f})")
            print(f"  🎯 Routing to {selected_venues} with {instruction.value}")
            
            round_fills = 0
            round_records = []
            
            for venue_name in selected_venues:
                if remaining_qty <= 0:
                    break
                
                # Calculate slice for this venue
                venue_liquidity = liquidity.get(venue_name, 0)
                slice_qty = min(remaining_qty, venue_liquidity)
                
                if slice_qty <= 0:
                    continue
                
                # Create SOR child order
                sor_order = self._create_sor_order(
                    algo_child, venue_name, slice_qty, instruction, timestamp
                )
                
                # Send to venue
                venue = self.venues[venue_name]
                response, filled_qty, fill_price = venue.execute_order(
                    sor_order, self.market, instruction
                )
                
                # Record execution
                execution_record = {
                    'timestamp': timestamp.isoformat(),
                    'order_id': sor_order['order_id'],
                    'parent_order_id': algo_child['order_id'],
                    'venue': venue_name,
                    'instruction': instruction.value,
                    'requested_qty': slice_qty,
                    'filled_qty': filled_qty,
                    'fill_price': fill_price,
                    'response': response.value,
                    'retry_count': retry_count,
                    'market_bid': bid,
                    'market_ask': ask,
                    'available_liquidity': venue_liquidity
                }
                
                # Update order based on response
                if response == VenueResponse.FILLED:
                    sor_order['state'] = 'FILLED'
                    sor_order['filled_quantity'] = filled_qty
                    sor_order['average_price'] = fill_price
                    print(f"    ✅ {venue_name}: FILLED {filled_qty:,} @ {fill_price:.2f}")
                    
                elif response == VenueResponse.PARTIAL:
                    sor_order['state'] = 'PARTIAL'
                    sor_order['filled_quantity'] = filled_qty
                    sor_order['average_price'] = fill_price
                    print(f"    ⚠️  {venue_name}: PARTIAL {filled_qty:,}/{slice_qty:,} @ {fill_price:.2f}")
                    
                elif response == VenueResponse.FADE:
                    sor_order['state'] = 'FADE'
                    sor_order['filled_quantity'] = 0
                    print(f"    ❌ {venue_name}: FADE - liquidity taken by competitor")
                    execution_record['fade_event'] = True
                    
                elif response == VenueResponse.NO_LIQUIDITY:
                    sor_order['state'] = 'NO_FILL'
                    sor_order['filled_quantity'] = 0
                    print(f"    ⚪ {venue_name}: NO LIQUIDITY")
                    
                else:  # REJECTED
                    sor_order['state'] = 'REJECTED'
                    sor_order['filled_quantity'] = 0
                    print(f"    ❌ {venue_name}: REJECTED")
                
                execution_records.append(execution_record)
                round_records.append(execution_record)
                
                if filled_qty > 0:
                    round_fills += filled_qty
                    remaining_qty -= filled_qty
                    total_filled += filled_qty
                
                # Capture snapshot
                self.tick_db.capture_snapshot(sor_order, f'SOR_{response.value}', timestamp, price)
                
                timestamp += timedelta(milliseconds=random.randint(10, 50))
            
            # Check if we made progress this round
            if round_fills == 0 and remaining_qty > 0:
                retry_count += 1
                print(f"  ⚠️  No fills this round - {remaining_qty:,} still needed")
            else:
                retry_count += 1  # Still count as attempt
        
        # Final status
        if total_filled >= algo_child['quantity']:
            print(f"  ✅ SOR: Completed - filled {total_filled:,}/{algo_child['quantity']:,}")
        else:
            print(f"  ⚠️  SOR: Incomplete - filled {total_filled:,}/{algo_child['quantity']:,} after {retry_count} attempts")
        
        return execution_records, total_filled, timestamp
    
    def _create_sor_order(self, parent: Dict, venue: str, quantity: int, 
                         instruction: OrderInstruction, timestamp: datetime) -> Dict:
        """Create SOR child order"""
        order = {
            'order_id': f"SOR_{self.order_counter:05d}",
            'parent_order_id': parent['order_id'],
            'client_order_id': parent['client_order_id'],
            'order_level': 3,
            'order_type': 'ROUTE',
            'ticker': parent['ticker'],
            'side': parent['side'],
            'quantity': quantity,
            'filled_quantity': 0,
            'remaining_quantity': quantity,
            'average_price': 0.0,
            'state': 'PENDING',
            'instruction': instruction.value,
            'venue': venue,
            'create_time': timestamp.isoformat(),
            'update_time': timestamp.isoformat()
        }
        self.order_counter += 1
        return order

# ============== SIMULATION ==============
class TickDatabase:
    """Simple tick database for recording"""
    def __init__(self):
        self.snapshots = []
        
    def capture_snapshot(self, order, event_type, timestamp, market_price):
        snapshot = copy.deepcopy(order)
        snapshot['snapshot_time'] = timestamp.isoformat()
        snapshot['event_type'] = event_type
        snapshot['market_price'] = market_price
        self.snapshots.append(snapshot)

def run_sor_simulation():
    """Run SOR simulation with fade and retry scenarios"""
    
    print("=" * 80)
    print("ENHANCED SOR SIMULATION - FADE & RETRY SCENARIOS")
    print("=" * 80)
    
    # Initialize
    market = EnhancedMarketSimulator()
    tick_db = TickDatabase()
    
    # Create venues with different fade probabilities
    venues = {
        'NYSE': EnhancedVenue('NYSE', fade_probability=0.05),
        'NASDAQ': EnhancedVenue('NASDAQ', fade_probability=0.1),
        'ARCA': EnhancedVenue('ARCA', fade_probability=0.15),
        'BATS': EnhancedVenue('BATS', fade_probability=0.2),
        'DARK': EnhancedVenue('DARK', fade_probability=0.02),  # Dark pools less fade
        'IEX': EnhancedVenue('IEX', fade_probability=0.25)  # Highest competition
    }
    
    sor = EnhancedSmartOrderRouter(tick_db, venues, market)
    
    # Simulate multiple algo child orders
    timestamp = datetime.now().replace(hour=9, minute=30, second=0, microsecond=0)
    
    test_orders = [
        {'order_id': 'ALGO_001', 'quantity': 5000, 'scenario': 'Normal'},
        {'order_id': 'ALGO_002', 'quantity': 8000, 'scenario': 'High Fade'},
        {'order_id': 'ALGO_003', 'quantity': 3000, 'scenario': 'Low Liquidity'}
    ]
    
    all_executions = []
    
    for test_order in test_orders:
        print(f"\n{'='*60}")
        print(f"SCENARIO: {test_order['scenario']} - {test_order['quantity']:,} shares")
        print(f"{'='*60}")
        
        # Adjust market conditions for scenario
        if test_order['scenario'] == 'High Fade':
            market.competition_level = 0.9  # High competition
        elif test_order['scenario'] == 'Low Liquidity':
            market.competition_level = 0.3
            # Reduce liquidity temporarily
        else:
            market.competition_level = 0.5
        
        # Create algo child order
        algo_child = {
            'order_id': test_order['order_id'],
            'client_order_id': 'C123456',
            'ticker': 'ASML.AS',
            'side': 'Buy',
            'quantity': test_order['quantity']
        }
        
        # Route through SOR
        executions, filled, timestamp = sor.route_order(algo_child, timestamp)
        
        # Store results
        for exec_record in executions:
            exec_record['scenario'] = test_order['scenario']
            all_executions.append(exec_record)
        
        timestamp += timedelta(seconds=5)
    
    # Export execution analysis
    with open('sor_execution_analysis.json', 'w') as f:
        json.dump(all_executions, f, indent=2, default=str)
    
    # Create summary
    print("\n" + "=" * 80)
    print("EXECUTION ANALYSIS SUMMARY")
    print("=" * 80)
    
    # Analyze fade events
    fade_events = [e for e in all_executions if e.get('response') == 'FADE']
    print(f"\n📊 Fade Events: {len(fade_events)}")
    for event in fade_events:
        print(f"  - {event['venue']}: Lost {event['requested_qty']:,} shares @ {event['timestamp'][11:19]}")
    
    # Analyze retries
    retry_events = [e for e in all_executions if e['retry_count'] > 0]
    print(f"\n🔄 Retry Attempts: {len(retry_events)}")
    
    # Success rate by venue
    print(f"\n🎯 Venue Performance:")
    venue_stats = {}
    for exec in all_executions:
        venue = exec['venue']
        if venue not in venue_stats:
            venue_stats[venue] = {'attempts': 0, 'fills': 0, 'fades': 0, 'volume': 0}
        
        venue_stats[venue]['attempts'] += 1
        if exec['response'] == 'FILLED':
            venue_stats[venue]['fills'] += 1
            venue_stats[venue]['volume'] += exec['filled_qty']
        elif exec['response'] == 'FADE':
            venue_stats[venue]['fades'] += 1
    
    for venue, stats in sorted(venue_stats.items()):
        fill_rate = (stats['fills'] / stats['attempts'] * 100) if stats['attempts'] > 0 else 0
        print(f"  {venue:8} - Fill Rate: {fill_rate:5.1f}% | Fades: {stats['fades']} | Volume: {stats['volume']:,}")
    
    print("\n✅ Analysis complete - see sor_execution_analysis.json for details")

if __name__ == "__main__":
    run_sor_simulation()