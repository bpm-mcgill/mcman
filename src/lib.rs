pub mod installer;

// leaving this here for future reference
    // let path = "/home/user/Documents/install_test/";
    // let client = Client::new();
    // let mut installer = ServerInstaller::new(client);

    // let versions = installer.get_versions().await?;
    // let latest_mc = versions.first().unwrap();
    // let compat = installer.get_loader_compat(latest_mc, LoaderType::Fabric).await?.unwrap();
    // let latest_fab = compat.first().unwrap();
    
    // let serv = ServerKind::Modded { mc: latest_mc.into(), loadertype: LoaderType::Fabric, version: latest_fab.into() };
    // let serv = ServerKind::Vanilla { mc: latest_mc.into() };
    // installer.install_server(serv, Path::new(path)).await?; 
