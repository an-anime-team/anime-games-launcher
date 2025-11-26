return {
    standard = 1,

    groups = function(source_platform: string, target_platform: string)
        return {
            {
                name = "dxvk-gplasync",
                title = "DXVK Gplasync"
            }
        }
    end,

    components = function(group: string)
        return {
            {
                name = "dxvk-gplasync-v2.5.1-2",
                title = "DXVK Gplasync v2.5.1-2"
            },
            {
                name = "dxvk-gplasync-v2.5.1-1",
                title = "DXVK Gplasync v2.5.1-1"
            },
            {
                name = "dxvk-gplasync-v2.5-1",
                title = "DXVK Gplasync v2.5-1"
            }
        }
    end,

    component = {
        get_status = function(component: string)
            return nil
        end,

        get_diff = function(component: string)
            return nil
        end,

        apply = function(component: string, launch_info: table)
            return nil
        end
    },

    settings = {
        standard = 1,

        settings = {
            {
                entries = {
                    name = "dxvk_async",
                    title = {
                        en = "Enable DXVK async rendering",
                        ru = "Включить асинхронный рендеринг DXVK"
                    },
                    entry = {
                        format = "switch",
                        default = true
                    }
                }
            }
        }
    }
}
