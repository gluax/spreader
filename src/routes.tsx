import { withRouter, Route } from 'react-router';
import React from "react";
import { Text, View } from "@nodegui/react-nodegui";

import Home from "./pages/Home";
import About from "./pages/About";
import Test from "./pages/Test";

const LocationDisplay = withRouter(({ location }) => (
  <Text data-testid="location-display">{location.pathname}</Text>
));

export default function AppRoutes() {
  return (
    <View>
        <Route exact path="/" component={Home} />
        <Route path= "/appsettings" component={Test} />
        <Route path= "/contentconfigs" component={Test} />
        <Route path= "/add" component={Test} />
        <Route path= "/read/:content" component={Test} />
        
        <Route path="/about" component={About} />
        <Route path="/test" component={Test} />
        <LocationDisplay />
    </View>
  );
}