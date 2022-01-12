# Day 14 - Automatic Coffee brewing with AlarmClock and TimeOracle
Today we will me making an AlarmClock component. It will be connected to a TimeOracle to know when it has to call the CoffeeMachine component.

## How to test
1. Reset your environment: `resim reset`
2. Create the default account: `resim new-account`
3. Build and publish the three blueprints on the ledger: `resim publish .`. Remember the package address.
4. Setup the CoffeeMachine component: `resim call-function [package_address] CoffeeMachine new`. Note the returned component address somewhere.
5. Setup the AlarmClock component to call the "make_coffee" method on the CoffeeMachine component on 2021-12-25 07:00:00. `resim call-function [package_address] AlarmClock new [coffee_machine_component_address] make_coffee 1640433600`. Take note of the returned ResourceDef and the two component addresses. The ResourceDef is your admin badge. The first component is the TimeOracle and the second is the AlarmClock

Normally, you would do a script that calls the `try_trigger` method on the AlarmClock component every x seconds. We will do it manually to keep the example simple.
6. Call `resim call-method [alarm_clock_component_address] try_trigger 1,[admin_badge_address]`. You should see the message "Not ready yet !"
7. Set the time to 2021-12-25 07:00:00 on the TimeOracle: `resim call-method [time_oracle] set_current_time 1640433600 1,[admin_badge_address]`
8. Call again `resim call-method [alarm_clock_component_address] try_trigger 1,[admin_badge_address]`. You should see the message "Brewing coffee !"