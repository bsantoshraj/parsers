filter {
  mutate {
    replace => {
      "type" => ""
      "time" => ""
      "specversion" => ""
      "id" => ""
      "data.message" => ""
      "data.eventName" => ""
      "data.compartmentId" => ""
      "data.compartmentName" => ""
      "data.eventGroupingId" => ""
      "data.idprimitive.ipAddress" => ""
      "data.idprimitive.principalId" => ""
      "data.idprimitive.principalName" => ""
      "data.idprimitive.userAgent" => ""
      "data.idprimitive.credentials" => ""
      "data.idprimitive.consoleSessionId" => ""
      "data.idprimitive.tenantId" => ""
      "data.request.action" => ""
      "data.request.path" => ""
      "data.request.headers.Accept.0" => ""
      "data.request.headers.Accept-Encoding.0" => ""
      "data.request.headers.Connection.0" => ""
      "data.request.headers.Origin.0" => ""
      "data.request.headers.Referer.0" => ""
      "data.request.headers.authorization.0" => ""
      "data.request.headers.opc-request-id.0" => ""
      "data.request.headers.sec-ch-ua-platform.0" => ""
      "data.request.parameters.param0.0" => ""
      "data.request.parameters.param1.0" => ""
      "data.response.headers.Content-Length.0" => ""
      "data.response.headers.Content-Type.0" => ""
      "data.response.headers.access-control-allow-methods.0" => ""
      "data.resourceId" => ""
      "data.response.status" => ""
      "metadata_event_type" => "GENERIC_EVENT"
      "oracle.compartmentid" => ""
      "oracle.tenantid" => ""
      "ip1" => ""
      "ip2" => ""
      "createdby" => ""
      "data.type" => ""
    }
  }


json {
    source => "message"
    array_function => "split_columns"
    on_error => "not_valid_json"
  }

  #storing the type into test_type
  mutate {
    replace => {
        "test_type" => "%{type}"
    }
  }

  grok {
        match => {
            "test_type" => [
                "com\\.oraclecloud\\.%{WORD:service}\\.(?<verb>Create|Email|Update|Delete|Attach|Detach|[Gg]et|[Ll]ist|Launch|Terminate|Modify|Interactive|Authentication|Instance|Add|Email|Initiate|Link|Mfa|Receive|Verify|Remove|Access|Factor|Federated|interactive|head|put|reencrypt|Capture|Change|Schedule|Support)(?<primitive>\\w+)"

            ]
        }
        overwrite => ["verb","service","primitive"]
        on_error => "zerror.grok100"
    }

  
# principal ip
  if [data][idprimitive][ipAddress] != "" {
    grok {
        match => {
          "data.idprimitive.ipAddress" => "%{IP:ip1},%{IP:ip2}"
        }
        overwrite => ["ip1", "ip2"]
        on_error => "grok_failure1"
    }
    if [grok_failure1] {
      mutate {
        merge => {
          "event.idm.read_only_udm.principal.ip" => "data.idprimitive.ipAddress"
        }
        on_error => "not_valid_ip"
      }
    }
    else {
      mutate {
        merge => {
          "event.idm.read_only_udm.principal.ip" => "ip2"
        }
        on_error => "not_valid_ip1"
      }
      mutate {
        merge => {
          "event.idm.read_only_udm.principal.ip" => "ip1"
        }
        on_error => "not_valid_ip2"
      }
    }
  }

 #principalId
  if [data][idprimitive][principalId] != "" {
    mutate {
      replace => {
        "event.idm.read_only_udm.principal.user.userid" => "%{data.idprimitive.principalId}"
      }
    }
  }

#principalName
if [data][idprimitive][principalName] != "" {
    mutate {
      replace => {
        "event.idm.read_only_udm.principal.user.user_display_name" => "%{data.idprimitive.principalName}"
      }
    }
    if [data][idprimitive][principalId] == "" {
        mutate {
            replace => {
                "event.idm.read_only_udm.principal.user.userid" => "%{data.idprimitive.principalName}"
            }
        }
    }
  }

 if [data][idprimitive][credentials] != "" {
    mutate {
      replace => {
        "credentials_label.key" => "credentials"
        "credentials_label.value" => "%{data.idprimitive.credentials}"
      }
      merge => {
        "event.idm.read_only_udm.principal.user.attribute.labels" => "credentials_label"
      }
    }
  }

  if [data][request][headers][sec-ch-ua-platform][0] =~ "(?i)Linux" {
    mutate {
      replace => {
        "event.idm.read_only_udm.principal.platform" => "LINUX"
      }
    }
  }
  else if [data][request][headers][sec-ch-ua-platform][0] =~ "(?i)windows" {
    mutate {
      replace => {
        "event.idm.read_only_udm.principal.platform" => "WINDOWS"
      }
    }
  }
  else if [data][request][headers][sec-ch-ua-platform][0] =~ "(?i)mac|ios" {
    mutate {
      replace => {
        "event.idm.read_only_udm.principal.platform" => "MAC"
      }
    }
  }


#target
# target
  if [data][request][path] != "" {
    mutate {
      replace => {
        "event.idm.read_only_udm.target.url" => "%{data.request.path}"
      }
    }
  }

  if [data][resourceId] != "" {
    mutate {
      replace => {
        "event.idm.read_only_udm.target.resource.product_object_id" => "%{data.resourceId}"
      }
    }

    if [type] =~ "listretentionrules" {
      grok {
        match => {
          "data.resourceId" => "/n/%{DATA:namespace}/b/%{DATA:bucketId}/retentionRules"
        }
        on_error => "grok_failure"
      }
      mutate {
        replace => {
          "event.idm.read_only_udm.target.resource.resource_type" => "STORAGE_BUCKET"
        }
      }

      if ![grok_failure] {
        mutate {
          replace => {
            "event.idm.read_only_udm.target.resource.product_object_id" => "%{bucketId}"
            "event.idm.read_only_udm.target.resource.name" => "%{data.resourceId}"
          }
        }

        mutate {
          replace => {
            "namespace_label.key" => "namespace"
            "namespace_label.value" => "%{namespace}"
          }
          merge => {
            "event.idm.read_only_udm.target.resource.attribute.labels" => "namespace_label"
          }
        }
      }
    }
  }


#network
# network
  if [data][idprimitive][userAgent] != "" {
    mutate {
      convert => {
        "data.idprimitive.userAgent" => "parseduseragent"
      }
    }
    mutate {
      rename => {
        "data.idprimitive.userAgent" => "event.idm.read_only_udm.network.http.parsed_user_agent"
      }
    }
  }

  if [data][idprimitive][consoleSessionId] != "" {
    mutate {
      rename => {
        "data.idprimitive.consoleSessionId" => "event.idm.read_only_udm.network.session_id"
      }
    }
  }

  if [data][request][action] != "" {
    mutate {
      replace => {
        "event.idm.read_only_udm.network.http.method" => "%{data.request.action}"
      }
    }
  }

  if [data][response][status] != "" {
    mutate {
      convert => {
        "data.response.status" => "integer"
      }
    }
    mutate {
      rename => {
        "data.response.status" => "event.idm.read_only_udm.network.http.response_code"
      }
    }
  }

#metadata
# metadata
  if [specversion] != "" {
    mutate {
      replace => {
        "event.idm.read_only_udm.metadata.product_version" => "%{specversion}"
      }
    }
  }

  if [type] != "" {
    mutate {
      replace => {
        "event.idm.read_only_udm.metadata.product_event_type" => "%{type}"
      }
    }
  }

  if [data][message] != "" {
    mutate {
      replace => {
        "event.idm.read_only_udm.metadata.description" => "%{data.message}"
      }
    }
  }

  if [id] != "" {
    mutate {
      replace => {
        "event.idm.read_only_udm.metadata.product_log_id" => "%{id}"
      }
    }
  }

  if [time] != "" {
    date {
      match => ["time", "ISO8601"]
    }
  }



# additional
  if [oracle][tenantid] != "" {
    mutate {
      replace => {
        "tenant_id_label.key" => "tenantId"
        "tenant_id_label.value.string_value" => "%{oracle.tenantid}"
      }
      merge => {
        "event.idm.read_only_udm.additional.fields" => "tenant_id_label"
      }
    } 
  }
  else if [data][idprimitive][tenantId] != "" {
    mutate {
      replace => {
        "tenant_id_label.key" => "tenantId"
        "tenant_id_label.value.string_value" => "%{data.idprimitive.tenantId}"
      }
      merge => {
        "event.idm.read_only_udm.additional.fields" => "tenant_id_label"
      }
    }
  }

  if [data][compartmentName] != "" {
    mutate {
      replace => {
        "compartment_name_field.key" => "compartmentName"
        "compartment_name_field.value.string_value" => "%{data.compartmentName}"
      }
      merge => {
        "event.idm.read_only_udm.additional.fields" => "compartment_name_field"
      }
    }
  }

  if [oracle][compartmentid] != "" {
    mutate {
      replace => {
        "compartment_id_field.key" => "compartmentId"
        "compartment_id_field.value.string_value" => "%{oracle.compartmentid}"
      }
      merge => {
        "event.idm.read_only_udm.additional.fields" => "compartment_id_field"
      }
    }
  }
  else if [data][compartmentId] != "" {
    mutate {
      replace => {
        "compartment_id_field.key" => "compartmentId"
        "compartment_id_field.value.string_value" => "%{data.compartmentId}"
      }
      merge => {
        "event.idm.read_only_udm.additional.fields" => "compartment_id_field"
      }
    }
  }

  if [data][eventGroupingId] != "" {
    mutate {
      replace => {
        "event_grouping_id_field.key" => "eventGroupingId"
        "event_grouping_id_field.value.string_value" => "%{data.eventGroupingId}"
      }
      merge => {
        "event.idm.read_only_udm.additional.fields" => "event_grouping_id_field"
      }
    }  
  }

  if [data][request][headers][Accept][0] != "" {
    mutate {
      replace => {
        "accept_field.key" => "Request Headers Accept"
        "accept_field.value.string_value" => "%{data.request.headers.Accept.0}"
      }
      merge => {
        "event.idm.read_only_udm.additional.fields" => "accept_field"
      }
    }
  }

  if [data][request][headers][Accept-Encoding][0] != "" {
    mutate {
      replace => {
        "accept_encoding_field.key" => "Request Headers Accept-Encoding"
        "accept_encoding_field.value.string_value" => "%{data.request.headers.Accept-Encoding.0}"
      }
      merge => {
        "event.idm.read_only_udm.additional.fields" => "accept_encoding_field"
      }
    }
  }

  if [data][request][headers][Connection][0] != "" {
    mutate {
      replace => {
        "connection_field.key" => "Request Headers Connection"
        "connection_field.value.string_value" => "%{data.request.headers.Connection.0}"
      }
      merge => {
        "event.idm.read_only_udm.additional.fields" => "connection_field"
      }
    }
  }

  if [data][request][headers][Origin][0] != "" {
    mutate {
      replace => {
        "origin_field.key" => "Request Headers Origin"
        "origin_field.value.string_value" => "%{data.request.headers.Origin.0}"
      }
      merge => {
        "event.idm.read_only_udm.additional.fields" => "origin_field"
      }
    }
  }

  if [data][request][headers][Referer][0] != "" {
    mutate {
      replace => {
        "referer_field.key" => "Request Headers Referer"
        "referer_field.value.string_value" => "%{data.request.headers.Referer.0}"
      }
      merge => {
        "event.idm.read_only_udm.additional.fields" => "referer_field"
      }
    }
  }

  if [data][request][headers][authorization][0] != "" {
    mutate {
      replace => {
        "authorization_field.key" => "Request Headers Authorization"
        "authorization_field.value.string_value" => "%{data.request.headers.authorization.0}"
      }
      merge => {
        "event.idm.read_only_udm.additional.fields" => "authorization_field"
      }
    }
  }

  if [data][request][headers][opc-request-id][0] != "" {
    mutate {
      replace => {
        "opc_request_id_field.key" => "Request Headers opc-request-id"
        "opc_request_id_field.value.string_value" => "%{data.request.headers.opc-request-id.0}"
      }
      merge => {
        "event.idm.read_only_udm.additional.fields" => "opc_request_id_field"
      }
    }
  }

  if [data][request][parameters][param0][0] != "" {
    mutate {
      replace => {
        "param0_field.key" => "Request parameters param0"
        "param0_field.value.string_value" => "%{data.request.parameters.param0.0}"
      }
      merge => {
        "event.idm.read_only_udm.additional.fields" => "param0_field"
      }
    }
  }

  if [data][request][parameters][param1][0] != "" {
    mutate {
      replace => {
        "param1_field.key" => "Request parameters param1"
        "param1_field.value.string_value" => "%{data.request.parameters.param1.0}"
      }
      merge => {
        "event.idm.read_only_udm.additional.fields" => "param1_field"
      }
    }
  }


  if [data][response][headers][Content-Length][0] != "" {
    mutate {
      replace => {
        "content_length_field.key" => "Response Headers Content-Length"
        "content_length_field.value.string_value" => "%{data.response.headers.Content-Length.0}"
      }
      merge => {
        "event.idm.read_only_udm.additional.fields" => "content_length_field"
      }
    }
  }

  if [data][response][headers][Content-Type][0] != "" {
    mutate {
      replace => {
        "content_type_field.key" => "Response Headers Content-Type"
        "content_type_field.value.string_value" => "%{data.response.headers.Content-Type.0}"
      }
      merge => {
        "event.idm.read_only_udm.additional.fields" => "content_type_field"
      }
    }
  }

  if [data][response][headers][access-control-allow-methods][0] != "" {
    mutate {
      replace => {
        "allow_methods_field.key" => "Response Headers access-control-allow-methods"
        "allow_methods_field.value.string_value" => "%{data.response.headers.access-control-allow-methods.0}"
      }
      merge => {
        "event.idm.read_only_udm.additional.fields" => "allow_methods_field"
      }
    }
  }



#########
# my changes baked here
########

  if [service] == "idprimitiveSignOn" {
   if [verb] == "Delete" {
      mutate {
        replace => { "metadata.event_type" => "USER_LOGOUT" }
      }
 } else {
      mutate {
        replace => { "metadata.event_type" => "USER_LOGIN" }
     }
    }
}
  
#objectstorage
  if [service] == "objectstorage" {
   if [verb] == "get" {
      mutate {
         replace => { "metadata.event_type" => "RESOURCE_READ" }}
      }
   } else if [verb] == "list" {
      mutate {
         replace => { "metadata.event_type" => "RESOURCE_READ" }}
      }
   }  
   else if [service] == "objectstorage" and  [verb] == "put" {
      mutate {
         replace => { "metadata.event_type" => "RESOURCE_WRITE"
         }
      }
   } else if [service] == "objectstorage" and [verb] == "post" { 
      mutate {
         replace => { "metadata.event_type" => "RESOURCE_MODIFY" }
      }
   }  else if [service] == "objectstorage" and [verb] == "Create" {
       mutate {
           replace => {"metadata.event_type" => "RESOURCE_CREATE" }
       }
   } else if [service] == "objectstorage" and [verb] == "head" {
       mutate {
           replace => {"metadata.event_type" => "RESOURCE_READ"}
       }
   } else if [service] == "objectstorage" and [verb] == "recencrypt" {
       mutate {
          replace  => { "metadata.event_type" => "RESOURCE_OPERATION"}
       }
   } else if [service] == "objectstorage" and [verb] == "delete" {
       mutate {
          replace => {"objectstorage" => "RESOURCE_DELETION"}
       }
   } else if [service] == "objectstorage" and "verb" == "update"]  {
       mutate {
          replace  => { "metadata.event_type" => "RESOURCE_UPDATE" }
       }
   } else {
       mutate {
          replace => { "metadata.event_type" => "GENERIC_EVENT" }}
      }
   } 
 
   
   #natgateway
   if [service] == "natgateway" {
       if [verb] == "Get" or [verb] == "List" {
          mutate {
            replace => { "metadata.event_type" => "RESOURCE_READ" }
          }
       }
   }  

    #Streaming-ControlPlane
    if [service] == "Streaming-ControlPlane" {
        if [verb] == "Get" {
            mutate {
             replace => { "metadataevent_type" => "RESOURCE_READ"}
            }
        }    
    }

    #virtualNetwork
    if [service] == "virtualNetwork" {
    if [verb] == "Create" {
      mutate {
         replace => { "metadata.event_type" => "RESOURCE_CREATION" }
        }
    } else if [verb] == "List" {
      mutate {
         replace => { "metadata.event_type" => "RESOURCE_READ" }
         }
    }
    else if [verb] ==  "Get" {
      mutate {
         replace => { "metadata.event_type" => "RESOURCE_READ" }
        }
    }
    else if [verb] == "Attach" {
      mutate {
          replace => {"metadata.event_type" => "RESOURCE_OPERATION" }
       }
    }
    else if [verb] == "Detach" {
      mutate {
          replace => {"metadata.event_type" => "RESOURCE_OPERATION" }
       }
    }
    else if [verb] == "Delete" {
      mutate {
          replace => { "metadata.event_type" => "RESOURCE_DELETE" }
      }
     } 
    }

    if [service] == "computeApi" {
   if [verb] == "Attach" {
       mutate {
          replace => { "metadata.event_type" => "RESOURCE_OPERATION" }
       }
    }
      else if [verb] == "Get"  {
       mutate  {
          replace => { "metadata.event_type" => "RESOURCE_READ" }
       }
    }
    else if [verb] == "List" {
       mutate  {
          replace => { "metadata.event_type" => "RESOURCE_READ" }
       }
    }
      else if [verb] == "Update" {
        mutate {
          replace => { "metadata.event_type" => "RESOURCE_UPDATE" }
       }
      }
    }

#identityControlPlane
if [service] == "idprimitiveControlPlane" {
  if [verb] == "Create" {
    mutate {
      replace => { "metadata.event_type" => "USER_ACCOUNT_CHANGE" }
    }
  } else if [verb] == "Get"  {

    mutate {
      replace => { "metadata.event_type" => "RESOURCE_READ" }
    }
  } else if [verb] == 'List' {

    mutate {
      replace => { "metadata.event_type" => "RESOURCE_READ" }
    }
  }  else if [verb] == "Add" {
    mutate {
      replace => { "metadata.event_type" => "USER_CHANGE_PERMISSIONS" }
    }
  }
    else if [verb] == "Email" {
    mutate {
      replace => { "metadata.event_type" => "NOTIFICATION_STATUS" }
    }
  }
    else if [verb] == "Update" and [primitive] == "Device"  {
    mutate {
      replace => { "metadata.event_type" => "DEVICE_CONFIG_UPDATE" }
    }
  } 
    else if [verb] == "Update" and [primitive] == "Group"  {
    mutate {
      replace => { "metadata.event_type" => "GROUP_MODIFICATION" }
    }
  } else if [verb] == "Update" and [primitive] == "Group"  {
    mutate {
      replace => { "metadata.event_type" => "ssosettings" }
    }
  } else {
    mutate {
      replace => { "metadata.event_type" => "GENERIC_EVENT" }
    }
  }
}

#blockVolume
if [service] == "BlockVolumes" {
   if [verb] == "Create" {
      mutate {
         replace => { "metadata.event_type" => "RESOURCE_CREATE" }
      }
   }
   else if [verb] == "Get" {
      mutate {
         replace => { "metadata.event_type" => "RESOURCE_READ" }
         }
   }
   else if [verb] == "List" {
      mutate {
         replace => { "metadata.event_type" => "RESOURCE_READ" }
         }
   }
   else if [verb] == "Update" {
      mutate {
         replace => {"metadata.event_type" => "RESOURCE_UPDATE" }
      }
   }
   else if [verb] == "Delete" {
      mutate {
        replace =>  { "metadata.event_type" => "RESOURCE_DELETE" }
     }
   }
} 

# Authentication logic
  if [service] == "identityControlPlane" {
    if [verb] == "Create" {
      mutate {
        replace => { "metadata.event_type" => "USER_ACCOUNT_CHANGE" }
      }
    } else if [verb] == "Get" {
      mutate {
        replace => { "metadata.event_type" => "RESOURCE_READ" }
      }
    } else if [verb] == "List" {
      mutate {
        replace => { "metadata.event_type" => "RESOURCE_READ" }
      }
    } else if [verb] == "Add" {
      mutate {
        replace => { "metadata.event_type" => "USER_CHANGE_PERMISSIONS" }
      }
    } else if [verb] == "Email" {
      mutate {
        replace => { "metadata.event_type" => "NOTIFICATION_STATUS" }
      }
    } else if [verb] == "Update" and [primitive] == "Device" {
      mutate {
        replace => { "metadata.event_type" => "DEVICE_CONFIG_UPDATE" }
      }
    } else if [verb] == "Update" and [primitive] == "Group" {
      mutate {
        replace => { "metadata.event_type" => "GROUP_MODIFICATION" }
      }
    } else if [verb] == "Authentication" and [primitive] = "User" {
      mutate {
        replace => { "metadata.event_type" => "USER_LOGIN" }
      }
    } else {
      mutate {
        replace => { "metadata.event_type" => "GENERIC_EVENT" }
      }
    }
  }

  mutate {
    replace => {
      "event.idm.read_only_udm.metadata.product_name" => "OCI_AUDIT"
      "event.idm.read_only_udm.metadata.vendor_name" => "Oracle"
      #"event.idm.read_only_udm.metadata.event_type" => "%{metadata_event_type}"
      "event.idm.read_only_udm.metadata.event_type" => "%{metadata.event_type}"
    }
  }

 statedump{ label => "--------END--------"}
  mutate {
    merge => {
      "@output" => "event"
    }
  }
}
