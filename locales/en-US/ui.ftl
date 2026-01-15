ui_confirmation_yes = Yes
ui_confirmation_no = No
ui_confirmation_next = Next
ui_confirmation_back = Back
ui_confirmation_create = Create
ui_confirmation_stop = Stop

ui_menu_sidebar_market_tab = Market
ui_menu_sidebar_servers_section = Servers
ui_menu_sidebar_develop_section = Develop
ui_menu_sidebar_scripts_tab = Scripts
ui_menu_sidebar_black_hat_section = Black Hat
ui_menu_sidebar_exploit_tab = Exploit
ui_menu_sidebar_glossary_tab = Glossary

ui_window_tutorial_title = Tutorial

ui_server_thread_count = Threads: {$thread_count}
ui_server_clock_speed = { $unit ->
    [ghz] Speed: {NUMBER($clock_speed, minimumFractionDigits: 4)} GHz
    [mhz] Speed: {NUMBER($clock_speed, minimumFractionDigits: 4)} MHz
    *[other] Speed: {NUMBER($clock_speed, minimumFractionDigits: 4)} Hz
}

ui_algorithm_instruction_count = Instruction Count: {$instruction_count}
ui_algorithm_algorithms_header = Algorithms
ui_algorithm_effects_header = Effects
ui_algorithm_procedure_header = Procedure
ui_algorithm_scripts_header = Scripts