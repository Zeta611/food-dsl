module Main exposing (main)

import Browser
import Html exposing (Html, aside, div, h2, input, label, node, span, text)
import Html.Attributes exposing (class, property, step, type_, value)
import Html.Events exposing (onInput)
import Json.Encode as E


main : Program () Model Msg
main =
    Browser.element { init = init, view = view, update = update, subscriptions = subscriptions }



-- MODEL


type alias Params =
    { radius : Float
    , tube : Float
    }


type alias Model =
    { params : Params }


init : () -> ( Model, Cmd Msg )
init _ =
    ( { params = { radius = 10, tube = 3 } }, Cmd.none )



-- UPDATE


type Msg
    = SetRadius Float
    | SetTube Float


update : Msg -> Model -> ( Model, Cmd Msg )
update msg ({ params } as model) =
    case msg of
        SetRadius r ->
            ( { model | params = { params | radius = r } }, Cmd.none )

        SetTube t ->
            ( { model | params = { params | tube = t } }, Cmd.none )



-- VIEW


view : Model -> Html Msg
view model =
    div [ class "relative h-screen w-screen overflow-hidden bg-base-200" ]
        [ node "render-preview"
            [ class "absolute inset-0 block"
            , property "params" (encodeParams model.params)
            ]
            []
        , aside [ class "card card-sm absolute bottom-4 right-4 w-80 bg-base-100 border-base-300 border-2 text-base-content shadow-lg" ]
            [ div [ class "card-body gap-1" ]
                [ h2 [ class "card-title" ] [ text "Configuration" ]
                , slider "Radius" { min = 2, max = 20, step = 0.5 } model.params.radius SetRadius
                , slider "Tube" { min = 0.5, max = 8, step = 0.1 } model.params.tube SetTube
                ]
            ]
        ]


slider : String -> { min : Float, max : Float, step : Float } -> Float -> (Float -> Msg) -> Html Msg
slider name range current toMsg =
    label [ class "flex flex-col gap-1" ]
        [ span [ class "flex justify-between text-sm" ]
            [ text name, span [ class "text-base-content/70" ] [ text (String.fromFloat current) ] ]
        , input
            [ type_ "range"
            , class "range range-xs"
            , Html.Attributes.min (String.fromFloat range.min)
            , Html.Attributes.max (String.fromFloat range.max)
            , step (String.fromFloat range.step)
            , value (String.fromFloat current)
            , onInput (String.toFloat >> Maybe.withDefault current >> toMsg)
            ]
            []
        ]


encodeParams : Params -> E.Value
encodeParams { radius, tube } =
    E.object [ ( "radius", E.float radius ), ( "tube", E.float tube ) ]



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.none
