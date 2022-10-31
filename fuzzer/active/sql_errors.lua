SQLI_ERRORS = read(string.format("%s/txt/sqli_errs.txt",SCRIPT_PATH))

PAYLOADS = {
    "'123",
    "''123",
    "`123",
    "\")123",
    "\"))123",
    "`)123",
    "`))123",
    "'))123",
    "')123\"123",
    "[]123",
    "\"\"123",
    "'\"123",
    "\"'123",
    "\123",
}

local function send_report(url,parameter,payload,matching_error)
    NewReport:setName("SQL Injection")
    NewReport:setDescription("https://owasp.org/www-community/attacks/SQL_Injection")
    NewReport:setRisk("high")
    NewReport:setUrl(url)
    NewReport:setParam(parameter)
    NewReport:setAttack(payload)
    NewReport:setEvidence(matching_error)
end

function main(url) 
    local resp = http:send("GET",HttpMessage:getUrl())
    if resp.errors:GetErrorOrNil() then
        local log_msg = string.format("[SQLI_ERRORS] Connection Error: %s",new_url)
        log_error(log_msg)
        return
    end
    for payload_index, payload in pairs(PAYLOADS) do 
        local new_query = HttpMessage:setAllParams(payload)
        for parameter_name, full_url in pairs(new_query) do
            local resp = http:send("GET",full_url)
            local body = resp.body:GetStrOrNil()
            for sqlerror_match in SQLI_ERRORS:gmatch("[^\n]+") do
                    local match = is_match(sqlerror_match,body)
                    if ( match == false or match == nil) then
                            -- NOTHING
                    else
                        send_report(resp.url:GetStrOrNil(),parameter_name,payload,sqlerror_match)
                        Reports:addReport(NewReport)
                        break
                    end
            end
        end
    end
end
