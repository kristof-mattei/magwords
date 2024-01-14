declare module "axios/lib/adapters/http" {
    import type { Adapter } from "axios";

    const HttpAdapter: Adapter;

    export default HttpAdapter;
}
